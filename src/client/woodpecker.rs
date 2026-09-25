use std::fmt;
use std::time::Duration;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::error::{Result, WoodpeckerError, truncate};

/// Characters left unescaped in a path segment (RFC 3986 unreserved set).
const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Percent-encode a user-supplied value for use as a single URL path segment.
pub fn encode_segment(s: &str) -> String {
    utf8_percent_encode(s, SEGMENT).to_string()
}

/// HTTP client wrapper for the Woodpecker CI API
#[derive(Clone)]
pub struct WoodpeckerClient {
    http: Client,
    /// Server base URL without trailing slash (may include a root path)
    base: String,
    token: String,
}

impl fmt::Debug for WoodpeckerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WoodpeckerClient")
            .field("base", &self.base)
            .finish_non_exhaustive()
    }
}

/// A successful response body, tagged with the request that produced it.
pub struct Reply {
    request: String,
    body: String,
}

impl Reply {
    /// Whether the server returned no content (empty body or JSON `null`).
    pub fn is_empty(&self) -> bool {
        matches!(self.body.trim(), "" | "null")
    }

    /// Decode the body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.body).map_err(|e| WoodpeckerError::Decode {
            request: self.request.clone(),
            message: format!("{e}. Body: {}", truncate(&self.body, 500)),
        })
    }

    /// Decode the body as JSON, treating an empty body or `null` as `None`.
    pub fn json_opt<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.json().map(Some)
        }
    }
}

impl WoodpeckerClient {
    pub fn new(config: &Config) -> Result<Self> {
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        Ok(Self {
            http,
            base: config.server_url.as_str().trim_end_matches('/').to_string(),
            token: config.token.clone(),
        })
    }

    /// Build a request to an `/api` endpoint. `path` must start with `/` and
    /// any user-supplied segments must already be passed through [`encode_segment`].
    pub fn api(&self, method: Method, path: &str) -> RequestBuilder {
        self.http
            .request(method, format!("{}/api{}", self.base, path))
    }

    /// Build a request to a root-level system endpoint such as `/version` or `/healthz`.
    pub fn root(&self, method: Method, path: &str) -> RequestBuilder {
        self.http.request(method, format!("{}{}", self.base, path))
    }

    /// Authorize and send a request, turning non-2xx responses into errors.
    pub async fn send(&self, request: RequestBuilder) -> Result<Reply> {
        let request = request.bearer_auth(&self.token).build()?;
        let label = format!("{} {}", request.method(), request.url().path());
        tracing::debug!("{label}");

        let response = self.http.execute(request).await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(Reply {
                request: label,
                body,
            })
        } else {
            Err(WoodpeckerError::from_response(
                label,
                status.as_u16(),
                &body,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_path_segments() {
        assert_eq!(encode_segment("MY_SECRET-1.x"), "MY_SECRET-1.x");
        assert_eq!(encode_segment("a/b c?"), "a%2Fb%20c%3F");
    }
}
