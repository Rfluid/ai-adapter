use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AI Adapter",
        version = "0.1.0",
        description = "WAHA → AI agent adapter. Receives WAHA webhooks, calls the AI, and (optionally) replies."
    ),
    servers(
        (url = "http://localhost:8080", description = "Local dev")
    ),
    tags(
        (name = "webhooks", description = "WAHA webhook endpoints")
    ),
    // Handlers (paths)
    paths(
        crate::routes::waha::receive_waha,
    ),
    // Schemas used in requests/responses
    components(
        schemas(
            crate::models::waha::WahaWebhook,
            crate::models::ai::InputRequestDoc,
            crate::models::ai::LlmApiResponse,
            crate::models::common::ErrorMessage
        )
    )
)]
pub struct ApiDoc;
