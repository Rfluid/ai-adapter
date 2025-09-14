use crate::{
    config::Config,
    models::waha::{WahaSeen, WahaTextOut, WahaTyping},
};

pub async fn send_text_message(
    http: &reqwest::Client,
    cfg: &Config,
    payload: WahaTextOut,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/api/sendText")
        .map_err(|e| e.to_string())?;

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

pub async fn start_typing(
    http: &reqwest::Client,
    cfg: &Config,
    payload: WahaTyping,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/api/startTyping")
        .map_err(|e| e.to_string())?;

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

pub async fn stop_typing(
    http: &reqwest::Client,
    cfg: &Config,
    payload: WahaTyping,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/api/stopTyping")
        .map_err(|e| e.to_string())?;

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

pub async fn send_seen(
    http: &reqwest::Client,
    cfg: &Config,
    payload: WahaSeen,
) -> Result<(), String> {
    // Adjust path to your WAHA send-message endpoint
    let url = cfg
        .waha_base_url
        .join("/api/sendSeen")
        .map_err(|e| e.to_string())?;

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
