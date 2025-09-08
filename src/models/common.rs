// src/models/common.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IncomingMessage {
    Text {
        user_id: String,
        body: String,
    },
    Unsupported {
        user_id: String,
        r#type: String,
        raw: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorMessage {
    pub error: String,
}
