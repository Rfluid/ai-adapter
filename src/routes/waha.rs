use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde_json::Value as JsonValue;
use tracing::info;

use crate::{AppState, handlers, models::waha::WahaWebhook};

#[utoipa::path(
    post,
    path = "/webhooks/waha",
    tag = "webhooks",
    params(
        ("x-allowed-wa-ids" = Option<String>, Header, description = "Comma-separated list of WhatsApp IDs to allow in dev mode.", example = "999999999999@c.us,111111111111@c.us"),
        ("x-typing" = Option<bool>, Header, description = "If `true`, typing indicators are allowed. Defaults to `true`.", example = true),
        ("x-send-seen" = Option<bool>, Header, description = "If `true`, sending read receipts (seen indicators) is allowed. Defaults to `true`.", example = true),
        ("x-ai-response" = Option<bool>, Header, description = "If `true`, AI-generated responses are allowed. Useful for development scenarios. Defaults to `true`.", example = true)
    ),
    request_body = WahaWebhook,
    responses(
        (status = 200, description = "Webhook accepted"),
        (status = 400, description = "Bad Request - Invalid webhook payload", body = crate::models::common::ErrorMessage),
        (status = 403, description = "Forbidden - WhatsApp ID not allowed", body = crate::models::common::ErrorMessage),
        (status = 500, description = "Handler error", body = crate::models::common::ErrorMessage)
    )
)]
pub async fn receive_waha(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<JsonValue>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Attempt to parse into our WAHA model; if it fails, return an error
    let webhook: WahaWebhook = serde_json::from_value(payload.clone()).map_err(|err| {
        // If deserialization fails, return an internal server error with the error message
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to deserialize webhook payload: {err}"),
        )
    })?;

    // Logic to handle development mode headers ---
    let allowed_wa_ids: Option<Vec<String>>;
    let mut typing: Option<bool> = Some(true);
    if let Some(will_type) = headers.get("x-typing") {
        if let Ok(will_type_str) = will_type.to_str() {
            typing = Some(will_type_str == "true");
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Header 'x-typing' contains invalid characters.".to_string(),
            ));
        }
    }

    // In dev mode, the x-allowed-wa-ids header is required
    if let Some(ids_header) = headers.get("x-allowed-wa-ids") {
        if let Ok(ids_str) = ids_header.to_str() {
            // Parse the comma-separated string into a Vec<String>
            // This handles whitespace and removes empty entries.
            let ids = ids_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>();

            info!("Dev mode: allowing WA IDs: {:?}", ids);
            allowed_wa_ids = Some(ids);
        } else {
            // Header value is not valid UTF-8 string
            return Err((
                StatusCode::BAD_REQUEST,
                "Header 'x-allowed-wa-ids' contains invalid characters.".to_string(),
            ));
        }
    } else {
        allowed_wa_ids = None;
    }

    info!(
        "Incoming WAHA webhook (id={} event={})",
        webhook.id, webhook.event,
    );

    // Normalize and dispatch
    handlers::dispatch_waha(webhook, state, allowed_wa_ids, typing)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("handler error: {e}"),
            )
        })?;

    // WAHA expects 200 quickly
    Ok(StatusCode::OK)
}
