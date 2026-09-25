# CLAUDE.md

A Rust MCP server (rmcp, stdio) that wraps the Woodpecker CI 3.x REST API. See README.md
for the tools and configuration.

## Layout

- `src/server.rs`: tool definitions (`#[tool_router]`), request types, and output shaping
- `src/client/woodpecker.rs`: HTTP client. `api()` and `root()` build requests; `send()` authorizes the request and maps HTTP errors
- `src/logs.rs`: decoding and windowing of step logs; `src/models.rs`: the few typed responses
- `src/error.rs`: `WoodpeckerError`, which turns into a tool result with `isError` set via `IntoCallToolResult`

## Conventions

- **Check API facts against Woodpecker's source at the server's version**, not memory or
  old docs. Routes are in `server/router/api.go`, JSON shapes in `server/model/*.go`, and
  response codes in `server/api/*.go`. The internal server runs 3.18. Field names changed
  between 2.x and 3.x (`created_at` became `created`, `error` became `errors`, and so on).
- **Pass responses through as `serde_json::Value`.** Add a typed struct only when the server
  itself reads fields, and give every field in it a default. Exact models have broken
  before: a string task ID was typed as i64, and 3.x fields were silently dropped.
- Tools return `Result<CallToolResult, WoodpeckerError>`. Do not return JSON-RPC errors for API failures.
- Pass user-supplied path segments (secret names, `owner/repo`) through `encode_segment`.
- Never log request bodies or tokens. Secret values pass through this server.
- Give every tool annotations: `read_only_hint`, and `destructive_hint` / `idempotent_hint`
  on tools that write.
- Keep `#[tool_handler(router = self.tool_router)]`. Without the argument, rmcp rebuilds
  the router on every request.

## Checks

```sh
cargo clippy --all-targets -- -D warnings
cargo test
```

CI (`.woodpecker/`) runs only on `v*` tags: it runs the check, then builds and uploads
release binaries to Gitea. The tumbletrove plugin's bootstrap downloads these binaries by
exact name (`woodpecker-ci-mcp-v<version>-{linux-x86_64,darwin-universal,windows-x86_64.exe}`),
so do not rename artifacts without updating the plugin. Keep CHANGELOG.md up to date under `[Unreleased]`.
