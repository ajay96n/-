use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use base64::Engine;
use crate::lobby::Lobby;
use crate::region::RegionInfo;

#[derive(Debug, Clone)]
pub struct LcuClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

#[derive(Debug, Deserialize)]
pub struct ReadyCheck {
    #[serde(rename = "declinerIds")]
    pub decliner_ids: Vec<Value>,
    #[serde(rename = "playerResponse")]
    pub player_response: Option<String>,
    pub state: String,
    #[serde(rename = "suppressUx")]
    pub suppress_ux: bool,
    pub timer: f64,
}

impl LcuClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (port, password) = Self::get_lockfile_info()?;
        
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()?;
        
        let base_url = format!("https://127.0.0.1:{}", port);
        let auth_header = format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(format!("riot:{}", password)));
        
        Ok(LcuClient {
            client,
            base_url,
            auth_header,
        })
    }
    
    fn get_lockfile_info() -> Result<(u16, String), Box<dyn std::error::Error>> {
        // Try common League of Legends installation paths
        let possible_paths = vec![
            "C:/Riot Games/League of Legends/lockfile",
            "/Applications/League of Legends.app/Contents/LoL/lockfile",
            "~/.local/share/Riot Games/League of Legends/lockfile",
        ];
        
        for path_str in possible_paths {
            let path = Path::new(path_str);
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let parts: Vec<&str> = content.trim().split(':').collect();
                
                if parts.len() >= 5 {
                    let port: u16 = parts[2].parse()?;
                    let password = parts[3].to_string();
                    return Ok((port, password));
                }
            }
        }
        
        Err("Could not find League of Legends lockfile".into())
    }
    
    pub async fn get_gameflow_phase(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/lol-gameflow/v1/gameflow-phase", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let phase: String = response.json().await?;
            Ok(phase.trim_matches('"').to_string())
        } else {
            Err("Failed to get gameflow phase".into())
        }
    }
    
    pub async fn get_ready_check(&self) -> Result<ReadyCheck, Box<dyn std::error::Error>> {
        let url = format!("{}/lol-matchmaking/v1/ready-check", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let ready_check: ReadyCheck = response.json().await?;
            Ok(ready_check)
        } else {
            Err("No ready check in progress".into())
        }
    }
    
    pub async fn accept_ready_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/lol-matchmaking/v1/ready-check/accept", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .json(&serde_json::json!({}))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err("Failed to accept ready check".into())
        }
    }
    
    pub async fn get_lobby_participants(&self) -> Result<Lobby, Box<dyn std::error::Error>> {
        let url = format!("{}/chat/v5/participants", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let lobby: Lobby = response.json().await?;
            Ok(lobby)
        } else {
            Err("Failed to get lobby participants".into())
        }
    }
    
    pub async fn get_region_info(&self) -> Result<RegionInfo, Box<dyn std::error::Error>> {
        let url = format!("{}/riotclient/region-locale", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
        
        if response.status().is_success() {
            let region_info: RegionInfo = response.json().await?;
            Ok(region_info)
        } else {
            Err("Failed to get region info".into())
        }
    }
}