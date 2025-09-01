mod analytics;
mod champ_select;
mod lobby;
mod region;
mod summoner;
mod utils;
mod lcu_client;

use crate::region::RegionInfo;
use crate::utils::display_champ_select;
use crate::lcu_client::{LCUClient, LCUClientInfo};
use std::time::Duration;
use tokio::sync::Mutex;

// Hardcoded configuration as requested
const AUTO_OPEN_MULTI: bool = true;
const AUTO_ACCEPT: bool = true;
const ACCEPT_DELAY: u32 = 2000;
const MULTI_PROVIDER: &str = "opgg"; // Fixed to op.gg as requested

struct LCUState {
    pub connected: bool,
    pub data: Option<LCUClientInfo>,
}

struct DodgeState {
    pub last_dodge: Option<u64>,
    pub enabled: Option<u64>,
}

#[tokio::main]
async fn main() {
    println!("League Reveal Console - Starting...");
    println!("Configuration:");
    println!("  Auto Open Multi: {}", AUTO_OPEN_MULTI);
    println!("  Auto Accept: {}", AUTO_ACCEPT);
    println!("  Accept Delay: {}ms", ACCEPT_DELAY);
    println!("  Multi Provider: {}", MULTI_PROVIDER);
    println!();

    let mut connected = false;
    let mut lcu_state = LCUState {
        connected: false,
        data: None,
    };
    let _dodge_state = Mutex::new(DodgeState {
        last_dodge: None,
        enabled: None,
    });

    loop {
        let lcu_info = match lcu_client::find_league_client().await {
            Some(info) => info,
            None => {
                if connected {
                    println!("Waiting for League Client to open...");
                    connected = false;
                    lcu_state.connected = false;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let app_client = match LCUClient::new(&lcu_info, false).await {
            Ok(client) => client,
            Err(e) => {
                println!("Failed to create app client: {:?}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let remoting_client = match LCUClient::new(&lcu_info, true).await {
            Ok(client) => client,
            Err(e) => {
                println!("Failed to create remoting client: {:?}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        connected = true;
        lcu_state.connected = true;
        lcu_state.data = Some(lcu_info);

        println!("Connected to League Client!");

        // Handle initial state
        let state = get_gameflow_state(&remoting_client).await;
        handle_client_state(state, &remoting_client, &app_client).await;

        // Simple polling loop instead of websockets for now
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Check if client is still running
            if lcu_client::find_league_client().await.is_none() {
                println!("League Client disconnected");
                break;
            }

            // Check gameflow state
            let state = get_gameflow_state(&remoting_client).await;
            handle_client_state(state, &remoting_client, &app_client).await;
        }
    }
}

async fn get_gameflow_state(remoting_client: &LCUClient) -> String {
    match remoting_client.get("/lol-gameflow/v1/gameflow-phase").await {
        Ok(response) => {
            let text = response.text().await.unwrap_or_default();
            text.replace('\"', "")
        }
        Err(_) => "Unknown".to_string(),
    }
}

async fn handle_client_state(
    client_state: String,
    remoting_client: &LCUClient,
    app_client: &LCUClient,
) {
    match client_state.as_str() {
        "ChampSelect" => {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            handle_champ_select_start(app_client, remoting_client).await;
        }
        "ReadyCheck" => {
            if AUTO_ACCEPT {
                tokio::time::sleep(std::time::Duration::from_millis(
                    (ACCEPT_DELAY as u64) - 1000,
                ))
                .await;
                let _resp = remoting_client
                    .post(
                        "/lol-matchmaking/v1/ready-check/accept",
                        serde_json::json!({}),
                    )
                    .await;
                println!("Auto-accepted ready check");
            }
        }
        _ => {}
    }

    println!("Client State Update: {}", client_state);
}

async fn handle_champ_select_start(
    app_client: &LCUClient,
    remoting_client: &LCUClient,
) {
    let team = lobby::get_lobby_info(app_client).await;
    let region_info: RegionInfo = match app_client.get("/riotclient/region-locale").await {
        Ok(response) => {
            match response.json().await {
                Ok(info) => info,
                Err(_) => {
                    println!("Failed to parse region info");
                    return;
                }
            }
        }
        Err(_) => {
            println!("Failed to get region info");
            return;
        }
    };

    println!("Champ select started!");

    if AUTO_OPEN_MULTI {
        let region = match region_info.web_region.as_str() {
            "SG2" => "SG",
            _ => &region_info.web_region,
        };

        display_champ_select(&team, region, MULTI_PROVIDER);
    }

    let summoner = summoner::get_current_summoner(remoting_client).await;
    analytics::send_analytics_event(&team, &summoner, &region_info).await;
}