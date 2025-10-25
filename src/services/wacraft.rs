use crate::config::WacraftConfig;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Clone)]
pub struct WacraftClient {
    http: reqwest::Client,
    config: Arc<RwLock<WacraftConfig>>,
}

impl WacraftClient {
    pub fn new(config: WacraftConfig, http: reqwest::Client) -> Self {
        Self {
            http,
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn send_text_message(&self, wa_id: &str, body: &str) -> Result<(), String> {
        let token = self.get_valid_token().await?;
        let contact = self
            .fetch_contact(&token, wa_id)
            .await?
            .ok_or_else(|| format!("Wacraft contact not found for wa_id {wa_id}"))?;

        let destination_wa_id = contact
            .product_details
            .as_ref()
            .and_then(|details| details.wa_id.clone())
            .unwrap_or_else(|| wa_id.to_string());

        let payload = SendMessageRequest {
            to_id: contact.id,
            sender_data: SenderData {
                messaging_product: "whatsapp".to_string(),
                recipient_type: Some("individual".to_string()),
                message_type: "text".to_string(),
                to: destination_wa_id,
                text: Some(TextBody {
                    body: body.to_string(),
                }),
            },
        };

        let url = {
            let cfg = self.config.read().await;
            cfg.base_url
                .join("message/whatsapp")
                .map_err(|err| format!("Failed to resolve Wacraft send endpoint: {err}"))?
        };

        let res = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|err| format!("Failed to send Wacraft message: {err}"))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "<body unavailable>".to_string());
            return Err(format!(
                "Wacraft send message failed with status {}: {}",
                status, body
            ));
        }

        Ok(())
    }

    pub async fn mark_message_as_read(
        &self,
        _wa_id: &str,
        _message_id: &str,
    ) -> Result<(), String> {
        debug!("mark_message_as_read is currently a no-op for Wacraft");
        Ok(())
    }

    async fn fetch_contact(
        &self,
        token: &str,
        wa_id: &str,
    ) -> Result<Option<MessagingProductContact>, String> {
        let mut url = {
            let cfg = self.config.read().await;
            cfg.base_url
                .join("messaging-product/contact/whatsapp")
                .map_err(|err| format!("Failed to resolve Wacraft contact endpoint: {err}"))?
        };

        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("wa_id", wa_id);
            pairs.append_pair("limit", "1");
        }

        let res = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|err| format!("Failed to query Wacraft contact: {err}"))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "<body unavailable>".to_string());
            return Err(format!(
                "Wacraft contact lookup failed with status {}: {}",
                status, body
            ));
        }

        let mut contacts: Vec<MessagingProductContact> = res
            .json()
            .await
            .map_err(|err| format!("Failed to parse Wacraft contact response: {err}"))?;

        Ok(contacts.pop())
    }

    async fn request_token(
        &self,
        base_url: &Url,
        request: TokenRequest,
    ) -> Result<TokenResponse, String> {
        let url = base_url
            .join("user/oauth/token")
            .map_err(|err| format!("Failed to resolve token endpoint: {err}"))?;

        let response = self
            .http
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|err| format!("Failed to request Wacraft token: {err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<body unavailable>".to_string());
            return Err(format!(
                "Token request failed with status {}: {}",
                status, body
            ));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|err| format!("Failed to parse token response: {err}"))
    }

    async fn get_valid_token(&self) -> Result<String, String> {
        let mut cfg = self.config.write().await;

        if let (Some(token), Some(expires_at)) = (&cfg.access_token, cfg.token_expires_at) {
            let now = current_timestamp()?;
            if expires_at > now + 60 {
                return Ok(token.clone());
            }
        }

        if let Some(refresh_token) = cfg.refresh_token.clone() {
            let request = TokenRequest {
                grant_type: "refresh_token".to_string(),
                username: None,
                password: None,
                refresh_token: Some(refresh_token),
            };
            match self.request_token(&cfg.base_url, request).await {
                Ok(response) => {
                    self.update_tokens(&mut cfg, response)?;
                    info!("Refreshed Wacraft access token via refresh_token");
                    if let Some(token) = &cfg.access_token {
                        return Ok(token.clone());
                    }
                }
                Err(err) => {
                    debug!(
                        "Failed to refresh Wacraft token with refresh_token: {}",
                        err
                    );
                }
            }
        }

        let request = TokenRequest {
            grant_type: "password".to_string(),
            username: Some(cfg.email.clone()),
            password: Some(cfg.password.clone()),
            refresh_token: None,
        };

        let response = self.request_token(&cfg.base_url, request).await?;
        self.update_tokens(&mut cfg, response)?;
        cfg.access_token
            .clone()
            .ok_or_else(|| "Wacraft token refresh failed".to_string())
    }

    fn update_tokens(
        &self,
        cfg: &mut WacraftConfig,
        response: TokenResponse,
    ) -> Result<(), String> {
        let now = current_timestamp()?;
        cfg.access_token = Some(response.access_token);
        cfg.refresh_token = Some(response.refresh_token);
        cfg.token_expires_at = Some(now + response.expires_in);
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct MessagingProductContact {
    id: String,
    #[serde(default)]
    product_details: Option<ProductDetails>,
}

#[derive(Debug, Deserialize)]
struct ProductDetails {
    #[serde(default)]
    wa_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TextBody {
    body: String,
}

#[derive(Debug, Serialize)]
struct SenderData {
    #[serde(rename = "messaging_product")]
    messaging_product: String,
    #[serde(rename = "recipient_type", skip_serializing_if = "Option::is_none")]
    recipient_type: Option<String>,
    #[serde(rename = "type")]
    message_type: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextBody>,
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    #[serde(rename = "to_id")]
    to_id: String,
    #[serde(rename = "sender_data")]
    sender_data: SenderData,
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    grant_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

fn current_timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| err.to_string())
        .map(|duration| duration.as_secs() as i64)
}
