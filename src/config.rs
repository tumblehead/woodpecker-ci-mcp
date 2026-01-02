use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the Woodpecker CI server
    pub server_url: String,
    /// Personal Access Token for authentication
    pub token: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_url = env::var("WOODPECKER_SERVER")
            .map_err(|_| ConfigError::MissingEnvVar("WOODPECKER_SERVER".to_string()))?;

        // Validate URL format
        if !server_url.starts_with("http://") && !server_url.starts_with("https://") {
            return Err(ConfigError::InvalidUrl(
                "WOODPECKER_SERVER must start with http:// or https://".to_string(),
            ));
        }

        let token = env::var("WOODPECKER_TOKEN")
            .map_err(|_| ConfigError::MissingEnvVar("WOODPECKER_TOKEN".to_string()))?;

        if token.is_empty() {
            return Err(ConfigError::MissingEnvVar(
                "WOODPECKER_TOKEN cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            server_url: server_url.trim_end_matches('/').to_string(),
            token,
        })
    }

    /// Get the API base URL (server_url + /api prefix)
    /// Note: System endpoints like /version and /healthz are at root level
    pub fn api_url(&self) -> String {
        format!("{}/api", self.server_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let config = Config {
            server_url: "https://ci.example.com".to_string(),
            token: "test-token".to_string(),
        };
        // Woodpecker API endpoints use /api prefix
        assert_eq!(config.api_url(), "https://ci.example.com/api");
    }

    #[test]
    fn test_trailing_slash_removed() {
        env::set_var("WOODPECKER_SERVER", "https://ci.example.com/");
        env::set_var("WOODPECKER_TOKEN", "test-token");

        let config = Config::from_env().unwrap();
        assert_eq!(config.server_url, "https://ci.example.com");

        env::remove_var("WOODPECKER_SERVER");
        env::remove_var("WOODPECKER_TOKEN");
    }
}
