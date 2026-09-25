use reqwest::Url;
use std::env;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("Invalid WOODPECKER_SERVER URL '{url}': {reason}")]
    InvalidUrl { url: String, reason: String },
}

#[derive(Clone)]
pub struct Config {
    /// Base URL of the Woodpecker CI server (including any root path)
    pub server_url: Url,
    /// Personal Access Token for authentication
    pub token: String,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("server_url", &self.server_url.as_str())
            .field("token", &"<redacted>")
            .finish()
    }
}

impl Config {
    /// Load configuration from the WOODPECKER_SERVER and WOODPECKER_TOKEN environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::new(
            env::var("WOODPECKER_SERVER").ok(),
            env::var("WOODPECKER_TOKEN").ok(),
        )
    }

    fn new(server: Option<String>, token: Option<String>) -> Result<Self, ConfigError> {
        let server = server
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(ConfigError::MissingEnvVar("WOODPECKER_SERVER"))?;

        // Tokens pasted from secret stores often carry a trailing newline
        let token = token
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .ok_or(ConfigError::MissingEnvVar("WOODPECKER_TOKEN"))?;

        let invalid = |reason: &str| ConfigError::InvalidUrl {
            url: server.clone(),
            reason: reason.to_string(),
        };

        let mut server_url = Url::parse(&server).map_err(|e| invalid(&e.to_string()))?;
        if !matches!(server_url.scheme(), "http" | "https") {
            return Err(invalid("scheme must be http or https"));
        }
        if server_url.query().is_some() || server_url.fragment().is_some() {
            return Err(invalid("must not contain a query or fragment"));
        }

        // A server-relative root path (WOODPECKER_ROOT_PATH) is kept; a trailing
        // "/api" is dropped because the client adds it itself.
        let path = server_url.path().trim_end_matches('/');
        let path = path.strip_suffix("/api").unwrap_or(path).to_string();
        server_url.set_path(&path);

        Ok(Self { server_url, token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(server: &str) -> Result<Config, ConfigError> {
        Config::new(Some(server.to_string()), Some("test-token".to_string()))
    }

    #[test]
    fn normalizes_server_url() {
        for input in [
            "https://ci.example.com",
            "https://ci.example.com/",
            "https://ci.example.com/api",
            "https://ci.example.com/api/",
            "  https://ci.example.com  ",
        ] {
            assert_eq!(
                config(input).unwrap().server_url.as_str(),
                "https://ci.example.com/",
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn keeps_root_path() {
        assert_eq!(
            config("https://example.com/ci/")
                .unwrap()
                .server_url
                .as_str(),
            "https://example.com/ci"
        );
    }

    #[test]
    fn rejects_invalid_urls() {
        assert!(config("ci.example.com").is_err());
        assert!(config("ftp://ci.example.com").is_err());
        assert!(config("https://ci.example.com/?x=1").is_err());
    }

    #[test]
    fn requires_non_empty_values() {
        assert!(matches!(
            Config::new(None, Some("t".into())),
            Err(ConfigError::MissingEnvVar("WOODPECKER_SERVER"))
        ));
        assert!(matches!(
            Config::new(Some("https://ci.example.com".into()), Some(" \n".into())),
            Err(ConfigError::MissingEnvVar("WOODPECKER_TOKEN"))
        ));
    }

    #[test]
    fn trims_token_and_redacts_debug() {
        let cfg = Config::new(
            Some("https://ci.example.com".into()),
            Some("secret-token\n".into()),
        )
        .unwrap();
        assert_eq!(cfg.token, "secret-token");
        assert!(!format!("{cfg:?}").contains("secret-token"));
    }
}
