use crate::{
    AppState,
    models::ai::{InputRequest, LlmApiResponse},
    services::{ai::send_user_message, waha::send_text_message},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextHandleError {
    #[error("ai call failed: {0}")]
    Ai(String),
    #[error("waha call failed: {0}")]
    Waha(String),
}

pub async fn handle_text(
    state: &AppState,
    session: &str,
    thread_id: &str,
    user_id: &str,
    body: &str,
) -> Result<(), TextHandleError> {
    let cfg = &state.cfg;

    let req = InputRequest {
        data: json!({
            "text": body,
            "source": "waha",
            "user_id": user_id,
        }),
        chat_interface: cfg.chat_interface.clone(),
        max_retries: cfg.max_retries,
        loop_threshold: cfg.loop_threshold,
        top_k: cfg.top_k,
        summarize_message_window: cfg.summarize_message_window,
        summarize_message_keep: cfg.summarize_message_keep,
        summarize_system_messages: cfg.summarize_system_messages,
        thread_id: thread_id.to_string(),
    };

    let ai_res: LlmApiResponse = send_user_message(&state.http, cfg, &req)
        .await
        .map_err(TextHandleError::Ai)?;

    if let Some(reply) = ai_res.response {
        // Only post back if AI intended to respond
        send_text_message(&state.http, cfg, session, user_id, &reply)
            .await
            .map_err(TextHandleError::Waha)?;
    }
    Ok(())
}

pub async fn handle_unsupported(
    state: &AppState,
    session: &str,
    thread_id: &str,
    user_id: &str,
    message_type: &str,
    raw: serde_json::Value,
) -> Result<(), TextHandleError> {
    let cfg = &state.cfg;
    let req = InputRequest {
        data: json!({
            "unsupported_message_type": message_type,
            "raw": raw,
            "source": "waha",
            "user_id": user_id,
        }),
        chat_interface: cfg.chat_interface.clone(),
        max_retries: cfg.max_retries,
        loop_threshold: cfg.loop_threshold,
        top_k: cfg.top_k,
        summarize_message_window: cfg.summarize_message_window,
        summarize_message_keep: cfg.summarize_message_keep,
        summarize_system_messages: cfg.summarize_system_messages,
        thread_id: thread_id.to_string(),
    };

    let ai_res: LlmApiResponse = send_user_message(&state.http, cfg, &req)
        .await
        .map_err(TextHandleError::Ai)?;

    if let Some(reply) = ai_res.response {
        send_text_message(&state.http, cfg, session, user_id, &reply)
            .await
            .map_err(TextHandleError::Waha)?;
    }
    Ok(())
}
