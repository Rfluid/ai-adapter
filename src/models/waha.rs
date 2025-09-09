use serde::{Deserialize, Serialize};
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

    pub raw: serde_json::Value,
}

/// A pragmatic WAHA view. WAHA variants differ; we store the minimum we need.
/// Adjust the From impl if your payload differs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WahaWebhook {
    pub id: String,
    pub session: String,
    pub event: String,

    pub payload: Option<WahaMessagePayload>,
    pub raw: serde_json::Value,
}
