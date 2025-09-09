use crate::config::Config;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct WahaTextOut {
    pub session: String,
    #[serde(rename = "chatId")]
    pub chat_id: String,
    #[serde(rename = "text")]
    pub text_body: String,
}

pub async fn send_text_message(
    http: &reqwest::Client,
    cfg: &Config,
    session: &str,
    chat_id: &str,
    text_body: &str,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/api/sendText")
        .map_err(|e| e.to_string())?;

    let payload = WahaTextOut {
        chat_id: chat_id.to_string(),
        text_body: text_body.to_string(),
        session: session.to_string(),
    };

    let mut req = http.post(url).json(&payload);
    if let Some(api_key) = &cfg.waha_api_key_plain {
        req = req.header("X-Api-Key", api_key);
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("request error: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("waha status {}", res.status()));
    }
    Ok(())
}
