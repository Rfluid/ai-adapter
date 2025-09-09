use axum::{Json, extract::State, http::StatusCode};
use serde_json::Value as JsonValue;
use tracing::info;

use crate::{AppState, handlers, models::waha::WahaWebhook};

#[utoipa::path(
    post,
    path = "/webhooks/waha",
    tag = "webhooks",
    request_body = WahaWebhook,
    responses(
        (status = 200, description = "Webhook accepted"),
        (status = 500, description = "Handler error", body = crate::models::common::ErrorMessage)
    )
)]
pub async fn receive_waha(
    State(state): State<AppState>,
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

    info!(
        "Incoming WAHA webhook (id={} event={})",
        webhook.id, webhook.event,
    );

    // Normalize and dispatch
    handlers::dispatch_waha(webhook, state).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("handler error: {e}"),
        )
    })?;

    // WAHA expects 200 quickly
    Ok(StatusCode::OK)
}
