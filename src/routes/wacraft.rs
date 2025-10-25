use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde_json::Value as JsonValue;
use tracing::info;

use crate::{AppState, handlers, models::wacraft::WacraftWebhook};

#[utoipa::path(
    post,
    path = "/webhooks/wacraft",
    tag = "webhooks",
    params(
        ("x-allowed-wa-ids" = Option<String>, Header, description = "Comma-separated list of WhatsApp IDs to allow in dev mode.", example = "999999999999@c.us,111111111111@c.us"),
        ("x-typing" = Option<bool>, Header, description = "If `true`, typing indicators are allowed. Defaults to `false` for Wacraft as the API does not provide native support.", example = false),
        ("x-send-seen" = Option<bool>, Header, description = "If `true`, attempts to mark messages as read. Currently a best-effort operation.", example = false),
        ("x-ai-response" = Option<bool>, Header, description = "If `true`, AI-generated responses are sent back through Wacraft. Defaults to `true`.", example = true)
    ),
    request_body = WacraftWebhook,
    responses(
        (status = 200, description = "Webhook accepted"),
        (status = 400, description = "Bad Request - Invalid webhook payload", body = crate::models::common::ErrorMessage),
        (status = 403, description = "Forbidden - WhatsApp ID not allowed", body = crate::models::common::ErrorMessage),
        (status = 500, description = "Handler error", body = crate::models::common::ErrorMessage)
    )
)]
pub async fn receive_wacraft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<JsonValue>,
) -> Result<StatusCode, (StatusCode, String)> {
    let webhook: WacraftWebhook = serde_json::from_value(payload.clone()).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to deserialize webhook payload: {err}"),
        )
    })?;

    let allowed_wa_ids = parse_allowed_ids(&headers)?;
    let typing = parse_bool_header(&headers, "x-typing")?.unwrap_or(false);
    let send_seen = parse_bool_header(&headers, "x-send-seen")?.unwrap_or(false);
    let ai_response = parse_bool_header(&headers, "x-ai-response")?.unwrap_or(true);

    info!("Incoming Wacraft webhook (id={})", webhook.id);

    handlers::dispatch_wacraft(
        webhook,
        state,
        allowed_wa_ids,
        typing,
        send_seen,
        ai_response,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("handler error: {e}"),
        )
    })?;

    Ok(StatusCode::OK)
}

fn parse_allowed_ids(headers: &HeaderMap) -> Result<Option<Vec<String>>, (StatusCode, String)> {
    if let Some(ids_header) = headers.get("x-allowed-wa-ids") {
        let ids_str = ids_header.to_str().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Header 'x-allowed-wa-ids' contains invalid characters.".to_string(),
            )
        })?;

        let ids = ids_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        if !ids.is_empty() {
            return Ok(Some(ids));
        }
    }
    Ok(None)
}

fn parse_bool_header(headers: &HeaderMap, key: &str) -> Result<Option<bool>, (StatusCode, String)> {
    if let Some(value) = headers.get(key) {
        let value = value.to_str().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                format!("Header '{}' contains invalid characters.", key),
            )
        })?;

        match value {
            "true" | "1" => Ok(Some(true)),
            "false" | "0" => Ok(Some(false)),
            _ => Err((
                StatusCode::BAD_REQUEST,
                format!("Header '{}' must be 'true' or 'false'.", key),
            )),
        }
    } else {
        Ok(None)
    }
}
