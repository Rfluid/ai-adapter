use std::env;
use std::str::FromStr;

use dotenvy::dotenv;
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP bind host (e.g., 0.0.0.0)
    pub app_host: String,
    /// HTTP bind port (e.g., 8080)
    pub app_port: u16,

    /// WAHA base URL (e.g., http://localhost:3000)
    pub waha_base_url: Url,
    /// Optional WAHA token/header if your WAHA needs it
    pub waha_token: Option<String>,

    /// AI base URL (e.g., http://localhost:8000)
    pub ai_base_url: Url,
    /// Path for the AI endpoint that receives user messages
    /// Usually "/agent/messages/user"
    pub ai_messages_user_path: String,

    /// Thread prefix for WAHA conversations (env), combined with user’s wa_id.
    pub thread_prefix_waha: String,

    /// Agent knobs (read from env, forwarded to AI)
    pub chat_interface: String, // default "api"
    pub max_retries: u32,                // default 1
    pub loop_threshold: u32,             // default 3
    pub top_k: u32,                      // default 5
    pub summarize_message_window: u32,   // default 4
    pub summarize_message_keep: u32,     // default 6
    pub summarize_system_messages: bool, // default false
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(&'static str),
    #[error("Invalid URL for {name}: {value}")]
    InvalidUrl { name: &'static str, value: String },
    #[error("Invalid number for {name}: {value}")]
    InvalidNumber { name: &'static str, value: String },
    #[error("General error: {0}")]
    Other(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env if present
        let _ = dotenv();

        let app_host = env_or_default("APP_HOST", "0.0.0.0");
        let app_port = parse_or_default::<u16>("APP_PORT", 8080)?;

        let waha_base_url = parse_url_required("WAHA_BASE_URL")?;
        let waha_token = env::var("WAHA_TOKEN").ok();

        let ai_base_url = parse_url_required("AI_BASE_URL")?;
        let ai_messages_user_path = env_or_default("AI_MESSAGES_USER_PATH", "/agent/messages/user");

        let thread_prefix_waha = env_or_default("THREAD_PREFIX_WAHA", "waha:");

        // Agent knobs (match your agent’s docs)
        let chat_interface = env_or_default("CHAT_INTERFACE", "api");
        let max_retries = parse_or_default::<u32>("MAX_RETRIES", 1)?;
        let loop_threshold = parse_or_default::<u32>("LOOP_THRESHOLD", 3)?;
        let top_k = parse_or_default::<u32>("TOP_K", 5)?;
        let summarize_message_window = parse_or_default::<u32>("SUMMARIZE_MESSAGE_WINDOW", 4)?;
        let summarize_message_keep = parse_or_default::<u32>("SUMMARIZE_MESSAGE_KEEP", 6)?;
        let summarize_system_messages = parse_bool_or_default("SUMMARIZE_SYSTEM_MESSAGES", false)?;

        Ok(Self {
            app_host,
            app_port,
            waha_base_url,
            waha_token,
            ai_base_url,
            ai_messages_user_path,
            thread_prefix_waha,
            chat_interface,
            max_retries,
            loop_threshold,
            top_k,
            summarize_message_window,
            summarize_message_keep,
            summarize_system_messages,
        })
    }
}

/* --------------------------- helpers --------------------------- */

fn env_or_default(key: &'static str, default: &'static str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_or_default<T: FromStr>(key: &'static str, default: T) -> Result<T, ConfigError> {
    match env::var(key) {
        Ok(v) => v.parse::<T>().map_err(|_| ConfigError::InvalidNumber {
            name: key,
            value: v,
        }),
        Err(_) => Ok(default),
    }
}

fn parse_bool_or_default(key: &'static str, default: bool) -> Result<bool, ConfigError> {
    match env::var(key) {
        Ok(v) => {
            let vv = v.to_lowercase();
            match vv.as_str() {
                "1" | "true" | "yes" | "y" => Ok(true),
                "0" | "false" | "no" | "n" => Ok(false),
                _ => Err(ConfigError::Other(format!("Invalid bool for {key}: {v}"))),
            }
        }
        Err(_) => Ok(default),
    }
}

fn parse_url_required(key: &'static str) -> Result<Url, ConfigError> {
    let raw = env::var(key).map_err(|_| ConfigError::MissingVar(key))?;
    Url::parse(&raw).map_err(|_| ConfigError::InvalidUrl {
        name: key,
        value: raw,
    })
}
