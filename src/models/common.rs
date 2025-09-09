// src/models/common.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IncomingMessage {
    Text {
        #[serde(rename = "chatId")]
        chat_id: String,
        body: String,
        session: String,
    },
    Unsupported {
        #[serde(rename = "chatId")]
        chat_id: String,
        session: String,

        r#type: String,
        // raw: WahaWebhook,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorMessage {
    pub error: String,
}
