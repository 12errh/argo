use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::McpError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpAuth {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "oauth2")]
    OAuth2 {
        client_id: String,
        client_secret: Option<String>,
        token_url: String,
        scopes: Option<Vec<String>>,
    },
}

pub struct AuthManager {
    auth: McpAuth,
    cached_token: Option<String>,
    token_expiry: Option<chrono::DateTime<chrono::Utc>>,
    http_client: Client,
}

impl AuthManager {
    pub fn new(auth: McpAuth) -> Self {
        Self {
            auth,
            cached_token: None,
            token_expiry: None,
            http_client: Client::new(),
        }
    }

    pub async fn get_auth_header(&mut self) -> Result<Option<String>, McpError> {
        match &self.auth {
            McpAuth::None => Ok(None),
            McpAuth::Bearer { token } => Ok(Some(format!("Bearer {}", token))),
            McpAuth::OAuth2 { .. } => {
                if self.is_token_valid() {
                    return Ok(self.cached_token.as_ref().map(|t| format!("Bearer {}", t)));
                }
                self.refresh_oauth2_token().await?;
                Ok(self.cached_token.as_ref().map(|t| format!("Bearer {}", t)))
            }
        }
    }

    fn is_token_valid(&self) -> bool {
        match (&self.cached_token, &self.token_expiry) {
            (Some(_), Some(expiry)) => chrono::Utc::now() < *expiry,
            _ => false,
        }
    }

    async fn refresh_oauth2_token(&mut self) -> Result<(), McpError> {
        let (client_id, client_secret, token_url, scopes) = match &self.auth {
            McpAuth::OAuth2 {
                client_id,
                client_secret,
                token_url,
                scopes,
            } => (
                client_id.clone(),
                client_secret.clone(),
                token_url.clone(),
                scopes.clone().unwrap_or_default(),
            ),
            _ => return Ok(()),
        };

        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), "client_credentials".to_string());
        params.insert("client_id".to_string(), client_id);
        if let Some(secret) = client_secret {
            params.insert("client_secret".to_string(), secret);
        }
        if !scopes.is_empty() {
            params.insert("scope".to_string(), scopes.join(" "));
        }

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| McpError::OAuth2Error {
                reason: format!("Token request failed: {}", e),
            })?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.map_err(|e| McpError::OAuth2Error {
            reason: format!("Failed to parse token response: {}", e),
        })?;

        if !status.is_success() {
            return Err(McpError::OAuth2Error {
                reason: format!(
                    "Token request failed with status {}: {}",
                    status,
                    body.get("error_description")
                        .or_else(|| body.get("error"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error")
                ),
            });
        }

        let token = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::OAuth2Error {
                reason: "No access_token in response".to_string(),
            })?
            .to_string();

        let expires_in = body
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600);

        self.cached_token = Some(token);
        self.token_expiry = Some(chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_none() {
        let manager = AuthManager::new(McpAuth::None);
        assert!(manager.cached_token.is_none());
        assert!(manager.token_expiry.is_none());
    }

    #[test]
    fn test_bearer_auth() {
        let manager = AuthManager::new(McpAuth::Bearer {
            token: "test-token".to_string(),
        });
        assert!(manager.cached_token.is_none());
    }

    #[test]
    fn test_token_validity_expired() {
        let mut manager = AuthManager::new(McpAuth::OAuth2 {
            client_id: "id".to_string(),
            client_secret: None,
            token_url: "https://example.com/token".to_string(),
            scopes: None,
        });
        manager.cached_token = Some("token".to_string());
        manager.token_expiry = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(!manager.is_token_valid());
    }

    #[test]
    fn test_token_validity_valid() {
        let mut manager = AuthManager::new(McpAuth::OAuth2 {
            client_id: "id".to_string(),
            client_secret: None,
            token_url: "https://example.com/token".to_string(),
            scopes: None,
        });
        manager.cached_token = Some("token".to_string());
        manager.token_expiry = Some(chrono::Utc::now() + chrono::Duration::hours(1));
        assert!(manager.is_token_valid());
    }
}
