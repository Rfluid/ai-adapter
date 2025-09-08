use crate::{
    AppState,
    models::{common::IncomingMessage, waha::WahaWebhook},
    utils::thread_id_for_waha,
};
use text::TextHandleError;
use thiserror::Error;

pub mod text;

#[derive(Debug, Error)]
pub enum HandleError {
    // This enables `?` to lift TextHandleError into HandleError automatically
    #[error(transparent)]
    Text(#[from] TextHandleError),
}

pub async fn dispatch_waha(webhook: WahaWebhook, state: AppState) -> Result<(), HandleError> {
    let msg = match webhook.message_type.as_str() {
        "text" => {
            let body = webhook.text_body.unwrap_or_default();
            IncomingMessage::Text {
                user_id: webhook.user_id.clone(),
                body,
            }
        }
        other => IncomingMessage::Unsupported {
            user_id: webhook.user_id.clone(),
            r#type: other.to_string(),
            raw: webhook.raw.clone(),
        },
    };

    let thread_id = thread_id_for_waha(&state.cfg, &webhook.user_id);
    match msg {
        IncomingMessage::Text { user_id, body } => {
            text::handle_text(&state, &thread_id, &user_id, &body).await?;
        }
        IncomingMessage::Unsupported {
            user_id,
            r#type,
            raw,
        } => {
            // Send a structured unsupported message to AI so it can decide
            text::handle_unsupported(&state, &thread_id, &user_id, &r#type, raw).await?;
        }
    }
    Ok(())
}
