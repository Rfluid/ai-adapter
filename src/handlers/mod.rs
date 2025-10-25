use crate::{
    AppState,
    models::{
        common::IncomingMessage,
        wacraft::{WacraftInteractive, WacraftReceiverData, WacraftWebhook},
        waha::WahaWebhook,
    },
    utils::{thread_id_for_wacraft, thread_id_for_waha},
};
use chrono::Utc;
use text::TextHandleError;
use thiserror::Error;
use tracing::{debug, warn};

pub mod text;
pub mod wacraft;

#[derive(Debug, Error)]
pub enum HandleError {
    #[error(transparent)]
    Text(#[from] TextHandleError),
    #[error(transparent)]
    Wacraft(#[from] wacraft::WacraftHandleError),
    #[error("Event '{0}' not supported")]
    EventNotSupported(String),
    #[error("Payload is missing")]
    MissingPayload,
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

pub async fn dispatch_waha(
    webhook: WahaWebhook,
    state: AppState,
    allowed_wa_ids: Option<Vec<String>>,
    typing: bool,
    send_seen: bool,
    ai_response: bool,
) -> Result<(), HandleError> {
    let event = webhook.event;
    if event != "message" {
        return Err(HandleError::EventNotSupported(event));
    }

    let payload = webhook.payload.ok_or(HandleError::MissingPayload)?;

    if payload.from_me {
        return Ok(());
    }

    let (message_type, payload_body) = if !payload.has_media {
        if let Some(body) = payload.body.clone() {
            ("text".to_string(), Some(body))
        } else {
            ("media".to_string(), None)
        }
    } else {
        ("media".to_string(), None)
    };

    if payload_body.as_deref() == Some("") {
        return Ok(());
    }

    let chat_id = payload.from;
    if let Some(wa_ids) = allowed_wa_ids {
        if !wa_ids.contains(&chat_id) {
            warn!(
                "DEV MODE: Blocking message from '{}' as it's not in the allowed list: {:?}",
                chat_id, wa_ids
            );
            return Ok(());
        }
    }

    let session = webhook.session;
    let thread_id = thread_id_for_waha(&state.cfg, &chat_id);
    let timestamp = payload.timestamp;
    let message_id = payload.id;

    let msg = match message_type.as_str() {
        "text" => IncomingMessage::Text {
            chat_id,
            session,
            timestamp,
            body: payload_body.unwrap_or_default(),
        },
        other => IncomingMessage::Unsupported {
            chat_id,
            session,
            timestamp,
            r#type: other.to_string(),
        },
    };

    match msg {
        IncomingMessage::Text {
            chat_id,
            session,
            body,
            timestamp,
        } => {
            text::handle_text(
                &state,
                &session,
                &thread_id,
                &chat_id,
                &message_id,
                &body,
                timestamp,
                typing,
                send_seen,
                ai_response,
            )
            .await?;
        }
        IncomingMessage::Unsupported {
            chat_id,
            session,
            timestamp,
            r#type,
        } => {
            text::handle_unsupported(
                &state,
                &session,
                &thread_id,
                &chat_id,
                &message_id,
                &r#type,
                timestamp,
                typing,
                send_seen,
                ai_response,
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn dispatch_wacraft(
    webhook: WacraftWebhook,
    state: AppState,
    allowed_wa_ids: Option<Vec<String>>,
    typing: bool,
    send_seen: bool,
    ai_response: bool,
) -> Result<(), HandleError> {
    let Some(receiver) = webhook.receiver_data else {
        debug!("Wacraft webhook without receiver_data, ignoring");
        return Ok(());
    };

    let chat_id = receiver
        .from
        .clone()
        .ok_or(HandleError::MissingField("receiver_data.from"))?;

    if let Some(ids) = allowed_wa_ids.as_ref() {
        if !ids.contains(&chat_id) {
            warn!(
                "DEV MODE: Blocking message from '{}' as it's not in the allowed list: {:?}",
                chat_id, ids
            );
            return Ok(());
        }
    }

    let message_id = receiver.id.clone().unwrap_or_else(|| webhook.id.clone());

    let session = webhook
        .messaging_product_id
        .clone()
        .or_else(|| webhook.from_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = receiver
        .timestamp
        .as_ref()
        .and_then(|ts| ts.parse::<i64>().ok())
        .unwrap_or_else(|| Utc::now().timestamp());

    let thread_id = thread_id_for_wacraft(&state.cfg, &chat_id);

    match normalize_wacraft_message(&receiver) {
        NormalizedMessage::Skip => Ok(()),
        NormalizedMessage::Text(body) => {
            wacraft::handle_text(
                &state,
                &session,
                &thread_id,
                &chat_id,
                &message_id,
                &body,
                timestamp,
                typing,
                send_seen,
                ai_response,
            )
            .await?;
            Ok(())
        }
        NormalizedMessage::Unsupported(kind) => {
            wacraft::handle_unsupported(
                &state,
                &session,
                &thread_id,
                &chat_id,
                &message_id,
                &kind,
                timestamp,
                typing,
                send_seen,
                ai_response,
            )
            .await?;
            Ok(())
        }
    }
}

enum NormalizedMessage {
    Text(String),
    Unsupported(String),
    Skip,
}

fn normalize_wacraft_message(data: &WacraftReceiverData) -> NormalizedMessage {
    let message_type = data.message_type.as_deref().unwrap_or("unknown");

    match message_type {
        "text" => {
            if let Some(body) = data.text.as_ref().and_then(|text| text.body.clone()) {
                if body.trim().is_empty() {
                    debug!(
                        "Skipping empty text body from {}",
                        data.from.as_deref().unwrap_or("<unknown>")
                    );
                    NormalizedMessage::Skip
                } else {
                    NormalizedMessage::Text(body)
                }
            } else {
                NormalizedMessage::Unsupported("text".to_string())
            }
        }
        "interactive" => {
            if let Some(body) = data.interactive.as_ref().and_then(interactive_to_body) {
                NormalizedMessage::Text(body)
            } else {
                let interactive_type = data
                    .interactive
                    .as_ref()
                    .and_then(|interactive| interactive.interactive_type.clone())
                    .unwrap_or_else(|| "interactive".to_string());
                NormalizedMessage::Unsupported(format!("interactive::{interactive_type}"))
            }
        }
        other => NormalizedMessage::Unsupported(other.to_string()),
    }
}

fn interactive_to_body(interactive: &WacraftInteractive) -> Option<String> {
    if let Some(list) = interactive.list_reply.as_ref() {
        let mut parts = Vec::new();
        if let Some(title) = &list.title {
            parts.push(title.clone());
        }
        if let Some(id) = &list.id {
            parts.push(format!("(id: {id})"));
        }
        if parts.is_empty() {
            return None;
        }
        let prefix = interactive
            .interactive_type
            .as_deref()
            .unwrap_or("list_reply");
        return Some(format!("[{prefix}] {}", parts.join(" ")));
    }

    if let Some(button) = interactive.button_reply.as_ref() {
        let mut parts = Vec::new();
        if let Some(title) = &button.title {
            parts.push(title.clone());
        }
        if let Some(id) = &button.id {
            parts.push(format!("(id: {id})"));
        }
        if parts.is_empty() {
            return None;
        }
        let prefix = interactive
            .interactive_type
            .as_deref()
            .unwrap_or("button_reply");
        return Some(format!("[{prefix}] {}", parts.join(" ")));
    }

    None
}
