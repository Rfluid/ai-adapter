use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRequest<T = serde_json::Value> {
    pub data: T,
    pub chat_interface: String,
    pub max_retries: u32,
    pub loop_threshold: u32,
    pub top_k: u32,
    pub summarize_message_window: u32,
    pub summarize_message_keep: u32,
    pub summarize_system_messages: bool,
    pub thread_id: String,
}

/// Doc-friendly, non-generic version for schema generation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InputRequestDoc {
    pub data: serde_json::Value,
    pub chat_interface: String,
    pub max_retries: u32,
    pub loop_threshold: u32,
    pub top_k: u32,
    pub summarize_message_window: u32,
    pub summarize_message_keep: u32,
    pub summarize_system_messages: bool,
    pub thread_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LlmApiResponse {
    pub next_step: String,
    pub next_step_reason: String,
    /// Optional in our tolerant runtime handling
    pub response: Option<String>,
}
