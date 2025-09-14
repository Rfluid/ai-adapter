use crate::{
    AppState,
    config::Config,
    models::{
        ai::{InputRequest, LlmApiResponse},
        waha::{WahaTextOut, WahaTyping},
    },
    services::{
        ai::send_user_message,
        waha::{send_text_message, start_typing, stop_typing},
    },
};
use chrono::{DateTime, Utc};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextHandleError {
    #[error("ai call failed: {0}")]
    Ai(String),
    #[error("waha call failed: {0}")]
    Waha(String),
}

// A "scope guard" to ensure stop_typing is always called.
struct TypingGuard {
    http: reqwest::Client,
    cfg: Config,
    session: String,
    chat_id: String,
}

impl Drop for TypingGuard {
    fn drop(&mut self) {
        // We can't await in drop, so we spawn a new task.
        // This is "fire-and-forget," which is fine for a non-critical
        // action like clearing the "typing..." indicator.

        // Clone the necessary owned data to move it into the async block.
        let http = self.http.clone();
        let cfg = self.cfg.clone();
        let waha_typing_payload = WahaTyping {
            chat_id: self.chat_id.clone(),
            session: self.session.clone(),
        };

        tokio::spawn(async move {
            if let Err(e) = stop_typing(&http, &cfg, waha_typing_payload).await {
                // You can log the error here if needed, but you can't return it.
                eprintln!("Failed to stop typing indicator: {}", e);
            }
        });
    }
}

pub async fn handle_text(
    state: &AppState,
    session: &str,
    thread_id: &str,
    chat_id: &str,
    body: &str,
    timestamp: i64,
    typing: Option<bool>,
) -> Result<(), TextHandleError> {
    let cfg = &state.cfg;

    // --- Start Typing ---
    if let Some(will_type) = typing {
        if will_type {
            start_typing(
                &state.http,
                cfg,
                WahaTyping {
                    chat_id: chat_id.to_string(),
                    session: session.to_string(),
                },
            )
            .await
            .map_err(TextHandleError::Waha)?;
        }
    }

    // --- Create the guard right after starting to type ---
    // The `_` prefix silences the "unused variable" warning.
    // The guard's only purpose is to be dropped at the end of the scope.
    let _typing_guard = TypingGuard {
        http: state.http.clone(),
        cfg: state.cfg.clone(), // Assuming state.cfg is an Arc<Config>
        session: session.to_string(),
        chat_id: chat_id.to_string(),
    };

    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now());

    let req = InputRequest {
        data: json!({
            "text": body,
            "source": "waha",
            "chat_id": chat_id,
            "timestamp": timestamp,
            "datetime": datetime.to_string(),
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
        .map_err(TextHandleError::Ai)?; // If this fails, the guard is dropped here!

    if let Some(reply) = ai_res.response {
        send_text_message(
            &state.http,
            cfg,
            WahaTextOut {
                chat_id: chat_id.to_string(),
                text_body: reply,
                session: session.to_string(),
            },
        )
        .await
        .map_err(TextHandleError::Waha)?; // Or here!
    }

    Ok(()) // If successful, the guard is dropped here as the function returns.
}
pub async fn handle_unsupported(
    state: &AppState,
    session: &str,
    thread_id: &str,
    chat_id: &str,
    message_type: &str,
    // raw: WahaWebhook,
) -> Result<(), TextHandleError> {
    let cfg = &state.cfg;
    let req = InputRequest {
        data: json!({
            "unsupported_message_type": message_type,
            "source": "waha",
            "chat_id": chat_id,
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
        send_text_message(
            &state.http,
            cfg,
            WahaTextOut {
                chat_id: chat_id.to_string(),
                text_body: reply,
                session: session.to_string(),
            },
        )
        .await
        .map_err(TextHandleError::Waha)?;
    }
    Ok(())
}
