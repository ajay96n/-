use reqwest::Client;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LCUClientInfo {
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

pub struct LCUClient {
    client: Client,
    base_url: String,
}

impl LCUClient {
    pub async fn new(info: &LCUClientInfo, use_remoting: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let base_url = if use_remoting {
            format!("{}://127.0.0.1:{}/", info.protocol, info.port)
        } else {
            format!("{}://127.0.0.1:{}/", info.protocol, info.port)
        };

        Ok(LCUClient { client, base_url })
    }

    pub async fn get(&self, endpoint: &str) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint.trim_start_matches('/'));
        let response = self.client.get(&url).send().await?;
        Ok(response)
    }

    pub async fn post(&self, endpoint: &str, body: serde_json::Value) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint.trim_start_matches('/'));
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;
        Ok(response)
    }
}

pub async fn find_league_client() -> Option<LCUClientInfo> {
    let mut system = System::new_all();
    system.refresh_processes();

    for (_, process) in system.processes() {
        if process.name().contains("LeagueClient") {
            let cmd_line = process.cmd();
            if let Some(info) = parse_league_args(cmd_line) {
                return Some(info);
            }
        }
    }

    None
}

fn parse_league_args(cmd_line: &[String]) -> Option<LCUClientInfo> {
    let mut port = None;
    let mut password = None;
    let mut protocol = "https".to_string();

    for arg in cmd_line {
        if arg.starts_with("--app-port=") {
            port = arg.split('=').nth(1).and_then(|p| p.parse().ok());
        } else if arg.starts_with("--remoting-auth-token=") {
            password = arg.split('=').nth(1).map(|s| s.to_string());
        } else if arg.starts_with("--app-protocol=") {
            protocol = arg.split('=').nth(1).unwrap_or("https").to_string();
        }
    }

    if let (Some(port), Some(password)) = (port, password) {
        Some(LCUClientInfo {
            port,
            password,
            protocol,
        })
    } else {
        None
    }
}