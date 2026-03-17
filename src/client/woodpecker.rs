use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::config::Config;
use crate::error::{Result, WoodpeckerError};

/// HTTP client wrapper for the Woodpecker CI API
#[derive(Debug, Clone)]
pub struct WoodpeckerClient {
    client: Client,
    /// Base URL for API endpoints (includes /api prefix)
    base_url: String,
    token: String,
}

impl WoodpeckerClient {
    /// Create a new Woodpecker API client
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(WoodpeckerError::HttpError)?;

        Ok(Self {
            client,
            base_url: config.api_url(),
            token: config.token.clone(),
        })
    }

    /// Add authorization header to request
    fn authorize(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(&self.token)
    }

    /// Build full URL for an endpoint
    fn url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }

    /// Handle response and convert errors
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status().as_u16();

        if (200..300).contains(&status) {
            let body = response.text().await.map_err(WoodpeckerError::HttpError)?;
            serde_json::from_str(&body).map_err(|e| {
                WoodpeckerError::Other(format!(
                    "Failed to parse response: {}. Body: {}",
                    e,
                    if body.len() > 500 { &body[..500] } else { &body }
                ))
            })
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(WoodpeckerError::from_response(status, body))
        }
    }

    /// Handle response that returns plain text or empty body
    async fn handle_text_response(&self, response: Response) -> Result<String> {
        let status = response.status().as_u16();

        if (200..300).contains(&status) {
            response.text().await.map_err(WoodpeckerError::HttpError)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(WoodpeckerError::from_response(status, body))
        }
    }

    /// Handle response for endpoints that return no content
    async fn handle_empty_response(&self, response: Response) -> Result<()> {
        let status = response.status().as_u16();

        if (200..300).contains(&status) {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(WoodpeckerError::from_response(status, body))
        }
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self
            .authorize(self.client.get(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// GET request that may return null (treated as None)
    pub async fn get_optional<T: DeserializeOwned>(&self, endpoint: &str) -> Result<Option<T>> {
        let response = self
            .authorize(self.client.get(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        let status = response.status().as_u16();

        if (200..300).contains(&status) {
            let body = response.text().await.map_err(WoodpeckerError::HttpError)?;
            if body.trim() == "null" || body.trim().is_empty() {
                Ok(None)
            } else {
                serde_json::from_str(&body)
                    .map(Some)
                    .map_err(|e| {
                        WoodpeckerError::Other(format!(
                            "Failed to parse response: {}. Body: {}",
                            e,
                            if body.len() > 500 { &body[..500] } else { &body }
                        ))
                    })
            }
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(WoodpeckerError::from_response(status, body))
        }
    }

    /// GET request with query parameters
    pub async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        let response = self
            .authorize(self.client.get(self.url(endpoint)))
            .query(query)
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// GET request that returns plain text or empty body
    pub async fn get_text(&self, endpoint: &str) -> Result<String> {
        let response = self
            .authorize(self.client.get(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_text_response(response).await
    }

    /// GET request to root-level endpoint (bypasses /api prefix)
    /// Used for system endpoints like /version and /healthz
    pub async fn get_root<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        // Extract server URL by removing /api suffix from base_url
        let server_url = self.base_url.trim_end_matches("/api");
        let url = format!("{}{}", server_url, endpoint);

        let response = self
            .authorize(self.client.get(&url))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// GET root-level endpoint that returns plain text
    /// Used for system endpoints like /healthz that return non-JSON
    pub async fn get_root_text(&self, endpoint: &str) -> Result<String> {
        let server_url = self.base_url.trim_end_matches("/api");
        let url = format!("{}{}", server_url, endpoint);

        let response = self
            .authorize(self.client.get(&url))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_text_response(response).await
    }

    /// POST request with JSON body
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.url(endpoint);
        let body_json = serde_json::to_string(body).unwrap_or_default();
        tracing::debug!("POST {} with body: {}", url, body_json);

        let response = self
            .authorize(self.client.post(&url))
            .json(body)
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// POST request without body
    pub async fn post_empty<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self
            .authorize(self.client.post(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// POST request that returns no content
    pub async fn post_no_content(&self, endpoint: &str) -> Result<()> {
        let response = self
            .authorize(self.client.post(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_empty_response(response).await
    }

    /// PATCH request with JSON body
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let response = self
            .authorize(self.client.patch(self.url(endpoint)))
            .json(body)
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }

    /// DELETE request
    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        let response = self
            .authorize(self.client.delete(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_empty_response(response).await
    }

    /// DELETE request that returns a response
    pub async fn delete_with_response<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self
            .authorize(self.client.delete(self.url(endpoint)))
            .send()
            .await
            .map_err(WoodpeckerError::HttpError)?;

        self.handle_response(response).await
    }
}
