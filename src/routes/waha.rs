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
    // Parse into our WAHA model, but keep raw JSON if structure varies
    let webhook: WahaWebhook = match serde_json::from_value(payload.clone()) {
        Ok(x) => x,
        Err(_) => WahaWebhook::from_loose(payload.clone()), // permissive fallback
    };

    info!(
        "Incoming WAHA webhook (user_id={} type={})",
        webhook.user_id, webhook.message_type
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
