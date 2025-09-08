use crate::config::Config;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
struct WahaTextOut<'a> {
    #[serde(rename = "messaging_product")]
    messaging_product: &'a str, // WAHA often uses "whatsapp"
    to: &'a str,
    r#type: &'a str,
    text: serde_json::Value,
}

pub async fn send_text_message(
    http: &reqwest::Client,
    cfg: &Config,
    to_user: &str,
    body: &str,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/messages")
        .map_err(|e| e.to_string())?;

    let payload = WahaTextOut {
        messaging_product: "whatsapp",
        to: to_user,
        r#type: "text",
        text: json!({ "body": body }),
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
