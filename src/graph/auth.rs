use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use oauth2::{
    basic::BasicClient, devicecode::StandardDeviceAuthorizationResponse, AuthUrl, ClientId,
    DeviceAuthorizationUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::time::{sleep, Duration as TokioDuration};

const MICROSOFT_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MICROSOFT_DEVICE_AUTH_URL: &str =
    "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCache {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

pub struct GraphAuthenticator {
    client_id: String,
    token_cache_path: PathBuf,
}

impl GraphAuthenticator {
    pub fn new(client_id: String, token_cache_path: PathBuf) -> Self {
        Self {
            client_id,
            token_cache_path,
        }
    }

    /// Initiate OAuth2 device code flow - displays user code and verification URL
    pub async fn login(&self) -> Result<()> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            None,
            AuthUrl::new(MICROSOFT_AUTH_URL.to_string())?,
            Some(TokenUrl::new(MICROSOFT_TOKEN_URL.to_string())?),
        )
        .set_device_authorization_url(DeviceAuthorizationUrl::new(
            MICROSOFT_DEVICE_AUTH_URL.to_string(),
        )?);

        let details: StandardDeviceAuthorizationResponse = client
            .exchange_device_code()?
            .add_scope(Scope::new("Calendars.ReadWrite".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to request device code")?;

        println!("\n🔐 Microsoft Graph Authentication");
        println!("════════════════════════════════════");
        println!("1. Visit: {}", details.verification_uri().as_str());
        println!("2. Enter code: {}", details.user_code().secret());
        println!("════════════════════════════════════\n");
        println!("Waiting for you to complete authentication...");

        // Poll for token
        let token = client
            .exchange_device_access_token(&details)
            .request_async(
                oauth2::reqwest::async_http_client,
                tokio::time::sleep,
                Some(TokioDuration::from_secs(details.expires_in().as_secs())),
            )
            .await
            .context("Failed to exchange device code for token")?;

        // Save tokens
        let cache = TokenCache {
            access_token: token.access_token().secret().clone(),
            refresh_token: token.refresh_token().map(|t| t.secret().clone()),
            expires_at: Utc::now()
                + Duration::seconds(token.expires_in().map(|d| d.as_secs() as i64).unwrap_or(3600)),
        };

        self.save_token_cache(&cache)?;
        println!("✓ Authentication successful! Tokens saved.");

        Ok(())
    }

    /// Get valid access token (refresh if expired)
    pub async fn get_access_token(&self) -> Result<String> {
        let mut cache = self.load_token_cache()?;

        // Check if token is expired (with 5 min buffer)
        if cache.expires_at < Utc::now() + Duration::minutes(5) {
            if let Some(refresh_token) = &cache.refresh_token {
                cache = self.refresh_access_token(refresh_token).await?;
            } else {
                anyhow::bail!(
                    "Access token expired and no refresh token available. Run 'task oauth login'"
                );
            }
        }

        Ok(cache.access_token)
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenCache> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            None,
            AuthUrl::new(MICROSOFT_AUTH_URL.to_string())?,
            Some(TokenUrl::new(MICROSOFT_TOKEN_URL.to_string())?),
        );

        let token = client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to refresh access token")?;

        let cache = TokenCache {
            access_token: token.access_token().secret().clone(),
            refresh_token: token
                .refresh_token()
                .map(|t| t.secret().clone())
                .or_else(|| Some(refresh_token.to_string())),
            expires_at: Utc::now()
                + Duration::seconds(token.expires_in().map(|d| d.as_secs() as i64).unwrap_or(3600)),
        };

        self.save_token_cache(&cache)?;
        Ok(cache)
    }

    fn load_token_cache(&self) -> Result<TokenCache> {
        let content = std::fs::read_to_string(&self.token_cache_path).context(format!(
            "Failed to read token cache. Run 'task oauth login' first. Path: {:?}",
            self.token_cache_path
        ))?;

        let cache: TokenCache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    fn save_token_cache(&self, cache: &TokenCache) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.token_cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(cache)?;
        std::fs::write(&self.token_cache_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_token_cache_save_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("tokens.json");

        let cache = TokenCache {
            access_token: "test_access".to_string(),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Utc::now() + Duration::hours(1),
        };

        let auth = GraphAuthenticator::new("test_client".to_string(), cache_path.clone());
        auth.save_token_cache(&cache).unwrap();

        let loaded = auth.load_token_cache().unwrap();
        assert_eq!(loaded.access_token, "test_access");
        assert_eq!(loaded.refresh_token, Some("test_refresh".to_string()));
    }
}
