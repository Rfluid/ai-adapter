use crate::{
    AppState,
    models::ai::{InputRequest, LlmApiResponse},
    services::ai::send_user_message,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum WacraftHandleError {
    #[error("ai call failed: {0}")]
    Ai(String),
    #[error("wacraft api call failed: {0}")]
    Wacraft(String),
    #[error("wacraft client not configured")]
    NotConfigured,
}

pub async fn handle_text(
    state: &AppState,
    session: &str,
    thread_id: &str,
    chat_id: &str,
    message_id: &str,
    body: &str,
    timestamp: i64,
    typing: bool,
    send_seen: bool,
    ai_response: bool,
) -> Result<(), WacraftHandleError> {
    let cfg = &state.cfg;
    let _guard = state.mutex_swapper.lock(chat_id.to_string()).await;
    let client = state
        .wacraft_client
        .as_ref()
        .cloned()
        .ok_or(WacraftHandleError::NotConfigured)?;

    if send_seen {
        if let Err(err) = client.mark_message_as_read(chat_id, message_id).await {
            debug!("Failed to mark Wacraft message as read: {}", err);
        }
    }

    if typing {
        debug!("Typing indicators are not currently supported for Wacraft");
    }

    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now());

    let req = InputRequest {
        data: json!({
            "text": body,
            "chat_id": chat_id,
            "session": session,
            "timestamp": timestamp,
            "current_date": datetime.to_string(),
            "source": "wacraft",
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

    if ai_response {
        let ai_res: LlmApiResponse = send_user_message(&state.http, cfg, &req)
            .await
            .map_err(WacraftHandleError::Ai)?;

        if let Some(reply) = ai_res.response {
            client
                .send_text_message(chat_id, &reply)
                .await
                .map_err(WacraftHandleError::Wacraft)?;
        }
    }

    Ok(())
}

pub async fn handle_unsupported(
    state: &AppState,
    session: &str,
    thread_id: &str,
    chat_id: &str,
    message_id: &str,
    message_type: &str,
    timestamp: i64,
    typing: bool,
    send_seen: bool,
    ai_response: bool,
) -> Result<(), WacraftHandleError> {
    let cfg = &state.cfg;
    let _guard = state.mutex_swapper.lock(chat_id.to_string()).await;
    let client = state
        .wacraft_client
        .as_ref()
        .cloned()
        .ok_or(WacraftHandleError::NotConfigured)?;

    if send_seen {
        if let Err(err) = client.mark_message_as_read(chat_id, message_id).await {
            debug!("Failed to mark Wacraft message as read: {}", err);
        }
    }

    if typing {
        debug!("Typing indicators are not currently supported for Wacraft");
    }

    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now());

    let req = InputRequest {
        data: json!({
            "unsupported_message_type": message_type,
            "chat_id": chat_id,
            "session": session,
            "timestamp": timestamp,
            "current_date": datetime.to_string(),
            "source": "wacraft",
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

    if ai_response {
        let ai_res: LlmApiResponse = send_user_message(&state.http, cfg, &req)
            .await
            .map_err(WacraftHandleError::Ai)?;

        if let Some(reply) = ai_res.response {
            client
                .send_text_message(chat_id, &reply)
                .await
                .map_err(WacraftHandleError::Wacraft)?;
        }
    }

    Ok(())
}
