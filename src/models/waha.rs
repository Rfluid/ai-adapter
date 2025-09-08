use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// A pragmatic WAHA view. WAHA variants differ; we store the minimum we need.
/// Adjust the From impl if your payload differs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WahaWebhook {
    pub user_id: String,
    pub message_type: String,
    pub text_body: Option<String>,
    pub raw: serde_json::Value,
}

impl WahaWebhook {
    /// Lenient constructor from arbitrary JSON.
    pub fn from_loose(v: Value) -> Self {
        // best-effort extraction (common shapes: messages[0].from / text.body)
        let user_id = v
            .pointer("/messages/0/from")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let message_type = v
            .pointer("/messages/0/type")
            .and_then(|x| x.as_str())
            .unwrap_or("unsupported")
            .to_string();
        let text_body = v
            .pointer("/messages/0/text/body")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        Self {
            user_id,
            message_type,
            text_body,
            raw: v,
        }
    }
}
