use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftWebhook {
    pub id: String,
    pub from_id: Option<String>,
    pub messaging_product_id: Option<String>,
    pub receiver_data: Option<WacraftReceiverData>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftReceiverData {
    pub context: Option<WacraftContext>,
    pub timestamp: Option<String>,
    #[serde(rename = "type")]
    pub message_type: Option<String>,
    pub interactive: Option<WacraftInteractive>,
    pub text: Option<WacraftText>,
    pub id: Option<String>,
    pub from: Option<String>,
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftContext {
    pub forwarded: Option<bool>,
    pub frequently_forwarded: Option<bool>,
    pub from: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftInteractive {
    #[serde(rename = "type")]
    pub interactive_type: Option<String>,
    #[serde(rename = "list_reply")]
    pub list_reply: Option<WacraftListReply>,
    #[serde(rename = "button_reply")]
    pub button_reply: Option<WacraftButtonReply>,
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftListReply {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftButtonReply {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WacraftText {
    pub body: Option<String>,
    pub preview_url: Option<bool>,
}
