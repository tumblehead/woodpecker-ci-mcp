//! Decoding, cleaning and windowing of step logs for get_step_logs.

use base64::Engine;

use crate::models::LogEntry;

/// Lines returned when the caller does not ask for a specific window.
pub const DEFAULT_MAX_LINES: usize = 1000;
/// Page size when paginating with `offset` but no `limit`.
pub const DEFAULT_PAGE_SIZE: usize = 100;

/// Which part of the log to return.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogWindow {
    pub lines: Option<usize>,
    pub head: bool,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Decode base64 log entries into `(line number, text)` pairs, stripping ANSI
/// escape codes and carriage-return progress redraws.
pub fn decode(entries: &[LogEntry]) -> Vec<(usize, String)> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            let data = entry.data.as_ref()?;
            let text = match base64::engine::general_purpose::STANDARD.decode(data) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => data.clone(),
            };
            Some((entry.line.unwrap_or(i), clean_line(&text)))
        })
        .collect()
}

/// Render the selected window of `lines` with a header describing what was returned.
pub fn render(lines: &[(usize, String)], window: LogWindow) -> String {
    let total = lines.len();
    let mut note = "";

    let slice = if window.offset.is_some() || window.limit.is_some() {
        let start = window.offset.unwrap_or(0).min(total);
        let end = start
            .saturating_add(window.limit.unwrap_or(DEFAULT_PAGE_SIZE))
            .min(total);
        &lines[start..end]
    } else if let Some(n) = window.lines {
        if window.head {
            &lines[..n.min(total)]
        } else {
            &lines[total.saturating_sub(n)..]
        }
    } else if total > DEFAULT_MAX_LINES {
        note = "Output capped to the last lines by default; use offset/limit or lines/head to read other parts.\n";
        &lines[total - DEFAULT_MAX_LINES..]
    } else {
        lines
    };

    let header = match (slice.first(), slice.last()) {
        _ if slice.len() == total => format!("[{total} log lines total]\n"),
        (Some((first, _)), Some((last, _))) => {
            format!("[Log lines {first}-{last} of {total} total]\n{note}")
        }
        _ => format!("[No lines in the requested range; the log has {total} lines]\n"),
    };

    let mut out = header;
    out.push('\n');
    for (num, text) in slice {
        out.push_str(&format!("{num}: {text}\n"));
    }
    out.truncate(out.trim_end().len());
    out
}

/// Remove ANSI escape sequences and keep only the final carriage-return
/// segment, which is what a terminal would have displayed.
fn clean_line(text: &str) -> String {
    let text = text.trim_end_matches(['\n', '\r']);
    let text = text.rsplit('\r').next().unwrap_or(text);
    strip_ansi(text)
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.next() {
            // CSI: ESC [ params... final byte in @..~
            Some('[') => {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            // OSC: ESC ] ... terminated by BEL or ESC \
            Some(']') => {
                while let Some(c) = chars.next() {
                    if c == '\x07' || (c == '\x1b' && chars.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            // Any other two-character escape
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(line: usize, text: &str) -> LogEntry {
        LogEntry {
            line: Some(line),
            data: Some(base64::engine::general_purpose::STANDARD.encode(text)),
        }
    }

    fn numbered(n: usize) -> Vec<(usize, String)> {
        (0..n).map(|i| (i, format!("line {i}"))).collect()
    }

    #[test]
    fn decodes_and_cleans_lines() {
        let entries = [
            entry(0, "\x1b[1m\x1b[32m   Compiling\x1b[0m foo v0.1.0\n"),
            entry(1, "Downloading 10%\rDownloading 50%\rDone"),
            entry(2, "\x1b]8;;https://x\x07link\x1b]8;;\x07"),
            entry(3, "caf\u{e9}"),
            LogEntry {
                line: Some(4),
                data: None,
            },
        ];
        let lines = decode(&entries);
        assert_eq!(
            lines,
            vec![
                (0, "   Compiling foo v0.1.0".to_string()),
                (1, "Done".to_string()),
                (2, "link".to_string()),
                (3, "caf\u{e9}".to_string()),
            ]
        );
    }

    #[test]
    fn invalid_utf8_is_lossy_not_base64() {
        let entries = [LogEntry {
            line: Some(0),
            data: Some(base64::engine::general_purpose::STANDARD.encode([b'o', b'k', 0xff])),
        }];
        assert_eq!(decode(&entries)[0].1, "ok\u{fffd}");
    }

    #[test]
    fn returns_everything_when_small() {
        let out = render(&numbered(3), LogWindow::default());
        assert_eq!(
            out,
            "[3 log lines total]\n\n0: line 0\n1: line 1\n2: line 2"
        );
    }

    #[test]
    fn caps_large_logs_to_the_tail_by_default() {
        let out = render(&numbered(DEFAULT_MAX_LINES + 5), LogWindow::default());
        assert!(out.starts_with(&format!(
            "[Log lines 5-{} of {} total]",
            DEFAULT_MAX_LINES + 4,
            DEFAULT_MAX_LINES + 5
        )));
        assert!(out.contains("capped"));
    }

    #[test]
    fn head_and_tail() {
        let lines = numbered(10);
        let head = render(
            &lines,
            LogWindow {
                lines: Some(2),
                head: true,
                ..Default::default()
            },
        );
        assert!(head.starts_with("[Log lines 0-1 of 10 total]"));
        let tail = render(
            &lines,
            LogWindow {
                lines: Some(2),
                ..Default::default()
            },
        );
        assert!(tail.starts_with("[Log lines 8-9 of 10 total]"));
    }

    #[test]
    fn pagination_handles_out_of_range_and_huge_limits() {
        let lines = numbered(10);
        let page = render(
            &lines,
            LogWindow {
                offset: Some(8),
                limit: Some(usize::MAX),
                ..Default::default()
            },
        );
        assert!(page.starts_with("[Log lines 8-9 of 10 total]"));
        let past_end = render(
            &lines,
            LogWindow {
                offset: Some(50),
                ..Default::default()
            },
        );
        assert!(past_end.starts_with("[No lines in the requested range"));
    }
}
