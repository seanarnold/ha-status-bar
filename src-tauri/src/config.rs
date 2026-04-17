use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub ha_url: String,
    pub ha_token: String,
    pub selected_entities: Vec<String>,
    pub refresh_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ha_url: "http://homeassistant.local:8123".into(),
            ha_token: String::new(),
            selected_entities: vec![],
            refresh_interval_secs: 30,
        }
    }
}
