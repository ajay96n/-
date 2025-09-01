use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub auto_open: bool,
    pub auto_accept: bool,
    pub accept_delay: u32,
    pub multi_provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_open: true,
            auto_accept: true,
            accept_delay: 2000,
            multi_provider: "opgg".to_string(),
        }
    }
}