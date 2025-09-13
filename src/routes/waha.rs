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
        ("x-allowed-wa-ids" = Option<String>, Header, description = "Comma-separated list of WhatsApp IDs to allow in dev mode.")
    ),
    request_body = WahaWebhook,
    responses(
        (status = 200, description = "Webhook accepted"),
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
    handlers::dispatch_waha(webhook, state, allowed_wa_ids)
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
