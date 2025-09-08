use crate::{config::Config, models::ai::InputRequest};
use serde::de::DeserializeOwned;

pub async fn send_user_message<R: DeserializeOwned>(
    http: &reqwest::Client,
    cfg: &Config,
    body: &InputRequest,
) -> Result<R, String> {
    let url = cfg
        .ai_base_url
        .join(&cfg.ai_messages_user_path)
        .map_err(|e| e.to_string())?;
    let res = http
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("request error: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("ai status {}", res.status()));
    }
    res.json::<R>()
        .await
        .map_err(|e| format!("json error: {e}"))
}
