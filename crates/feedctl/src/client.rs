//! Talking to the feedd API, and turning its answers into [`CliError`]s.

use feedhub_core::api::ErrorBody;
use reqwest::{Client, Method, StatusCode};
use serde_json::Value;

use crate::CliError;

pub struct Api {
    base: String,
    client: Client,
}

impl Api {
    pub fn new(base: &str) -> Api {
        Api {
            base: base.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Make a request and return the response body.
    ///
    /// `204 No Content` and an empty body both come back as [`Value::Null`].
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value, CliError> {
        let url = format!("{}{path}", self.base);
        let mut request = self.client.request(method, &url).query(query);
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|e| {
            // Anything that stops the request from getting an answer at all is a
            // "server unreachable", which is exit code 2.
            CliError::unreachable(format!("cannot reach feedd at {}: {}", self.base, root(&e)))
        })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if status.is_success() {
            if status == StatusCode::NO_CONTENT || text.trim().is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_str(&text).map_err(|e| {
                CliError::unreachable(format!("feedd sent a response that is not JSON: {e}"))
            });
        }

        // The API's error shape; fall back to the raw body if it is something
        // else entirely.
        let message = serde_json::from_str::<ErrorBody>(&text)
            .map(|body| body.error)
            .unwrap_or_else(|_| {
                let text = text.trim();
                if text.is_empty() {
                    format!("server returned {status}")
                } else {
                    format!("server returned {status}: {text}")
                }
            });
        Err(CliError::server(message))
    }

    pub async fn get(&self, path: &str, query: &[(String, String)]) -> Result<Value, CliError> {
        self.request(Method::GET, path, query, None).await
    }

    pub async fn post(&self, path: &str, body: Option<Value>) -> Result<Value, CliError> {
        self.request(Method::POST, path, &[], body).await
    }

    pub async fn delete(&self, path: &str) -> Result<Value, CliError> {
        self.request(Method::DELETE, path, &[], None).await
    }
}

/// reqwest nests its causes; the innermost one is the part worth showing.
fn root(error: &reqwest::Error) -> String {
    let mut source: &dyn std::error::Error = error;
    while let Some(next) = source.source() {
        source = next;
    }
    source.to_string()
}
