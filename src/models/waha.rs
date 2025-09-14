use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WahaMessagePayload {
    pub id: String,
    pub timestamp: i64,

    pub from: String,
    pub to: String,

    pub body: Option<String>,

    #[serde(rename = "fromMe")]
    pub from_me: bool,
    #[serde(rename = "hasMedia")]
    pub has_media: bool,

    // Allow extra fields in a HashMap
    #[serde(flatten)]
    pub extra_fields: Option<HashMap<String, Value>>,
}

/// A pragmatic WAHA view. WAHA variants differ; we store the minimum we need.
/// Adjust the From impl if your payload differs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WahaWebhook {
    pub id: String,
    pub session: String,
    pub event: String,

    pub payload: Option<WahaMessagePayload>,
    pub extra_fields: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct WahaTyping {
    pub session: String,
    #[serde(rename = "chatId")]
    pub chat_id: String,
}

#[derive(Debug, Serialize)]
pub struct WahaSeen {
    pub session: String,
    #[serde(rename = "chatId")]
    pub chat_id: String,

    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
    pub participant: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WahaTextOut {
    pub session: String,
    #[serde(rename = "chatId")]
    pub chat_id: String,
    #[serde(rename = "text")]
    pub text_body: String,
}
