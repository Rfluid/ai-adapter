use crate::config::Config;

pub fn thread_id_for_waha(cfg: &Config, user_id: &str) -> String {
    format!("{}{}", cfg.thread_prefix_waha, user_id)
}
