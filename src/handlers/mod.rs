use crate::{
    AppState,
    models::{common::IncomingMessage, waha::WahaWebhook},
    utils::thread_id_for_waha,
};
use text::TextHandleError;
use thiserror::Error;
use tracing::warn;

pub mod text;

#[derive(Debug, Error)]
pub enum HandleError {
    // This enables `?` to lift TextHandleError into HandleError automatically
    #[error(transparent)]
    Text(#[from] TextHandleError),

    // Other error variants
    #[error("Event not supported")]
    EventNotSupported,

    // Payload error variant
    #[error("Payload is missing")]
    MissingPayload,
}

pub async fn dispatch_waha(
    webhook: WahaWebhook,
    state: AppState,
    allowed_wa_ids: Option<Vec<String>>,
    typing: bool,
) -> Result<(), HandleError> {
    // Check if it is a message event
    let event = webhook.event;
    if event != "message" {
        return Err(HandleError::EventNotSupported);
    }

    // Check if payload is None and return an error
    let payload = match webhook.payload {
        Some(p) => p,                                    // Successfully unwrapped the payload
        None => return Err(HandleError::MissingPayload), // Return error if payload is None
    };

    if payload.from_me {
        return Ok(()); // Ignore payload from user
    }

    let message_type;
    let payload_body: Option<String>;

    if !payload.has_media {
        if let Some(body) = payload.body {
            // Successfully extracted the body
            payload_body = Some(body);
            message_type = "text";
        } else {
            // If body is None, handle appropriately
            payload_body = None;
            message_type = "media";
        }
    } else {
        // Handle case when there is media
        payload_body = None;
        message_type = "media";
    }

    // Handle empty messages on first contact message
    if payload_body.as_deref() == Some("") {
        return Ok(());
    }

    let chat_id = payload.from;
    if let Some(wa_ids) = allowed_wa_ids {
        if !wa_ids.contains(&chat_id) {
            // If the chat_id is NOT in the allowed list, stop processing.
            warn!(
                "DEV MODE: Blocking message from '{}' as it's not in the allowed list: {:?}",
                chat_id, wa_ids
            );
            return Ok(()); // Exit early
        }
    }
    let session = webhook.session;

    let thread_id = thread_id_for_waha(&state.cfg, &chat_id);
    let timestamp = payload.timestamp;

    let msg = match message_type {
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
            // raw: webhook.clone(),
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
                &state, &session, &thread_id, &chat_id, &body, timestamp, typing,
            )
            .await?;
        }
        IncomingMessage::Unsupported {
            chat_id,
            session,
            timestamp,

            r#type,
            // raw,
        } => {
            // Send a structured unsupported message to AI so it can decide
            text::handle_unsupported(
                &state, &session, &thread_id, &chat_id, &r#type, timestamp, typing,
            )
            .await?;
        }
    }
    Ok(())
}
