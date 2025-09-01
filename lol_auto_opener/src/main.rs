mod config;
mod lcu_client;
mod lobby;
mod region;
mod utils;

use crate::config::Config;
use crate::lcu_client::LcuClient;
use crate::lobby::get_lobby_info;
use crate::utils::display_champ_select;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("🎮 League of Legends Auto Opener Console Application");
    println!("📋 Configuration:");
    println!("   • Auto Open Multi: ENABLED (OP.GG)");
    println!("   • Auto Accept: ENABLED");
    println!("   • Multi Provider: OP.GG (FIXED)");
    println!("🔄 Starting application...\n");

    let config = Config::default();
    let mut connected = false;
    let mut last_gameflow_state = String::new();

    loop {
        let client = match LcuClient::new().await {
            Ok(client) => client,
            Err(_) => {
                if connected {
                    println!("❌ League Client disconnected. Waiting for reconnection...");
                    connected = false;
                } else {
                    println!("⏳ Waiting for League Client to open...");
                }
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        if !connected {
            println!("✅ Connected to League Client!");
            connected = true;
        }

        // Check game flow state
        match client.get_gameflow_phase().await {
            Ok(current_state) => {
                if current_state != last_gameflow_state {
                    handle_client_state(&current_state, &config, &client).await;
                    last_gameflow_state = current_state;
                }
            }
            Err(_) => {
                connected = false;
                continue;
            }
        }

        // Check for ready check
        if config.auto_accept {
            if let Ok(ready_check) = client.get_ready_check().await {
                if ready_check.state == "InProgress" && !ready_check.player_response.is_some() {
                    println!("🎯 Ready check detected! Auto-accepting in {}ms...", config.accept_delay);
                    
                    sleep(Duration::from_millis(
                        (config.accept_delay as u64).saturating_sub(1000),
                    )).await;
                    
                    match client.accept_ready_check().await {
                        Ok(_) => println!("✅ Ready check accepted!"),
                        Err(e) => println!("❌ Failed to accept ready check: {}", e),
                    }
                }
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }
}

async fn handle_client_state(client_state: &str, config: &Config, client: &LcuClient) {
    match client_state {
        "ChampSelect" => {
            println!("🎯 Champion Select detected!");
            
            // Wait a bit for the champion select to fully load
            sleep(Duration::from_secs(5)).await;
            
            if let Err(e) = handle_champ_select_start(client, config).await {
                println!("❌ Error handling champion select: {}", e);
            }
        }
        "Lobby" => {
            println!("🏠 In lobby, waiting for queue...");
        }
        "Matchmaking" => {
            println!("🔍 Searching for match...");
        }
        "ReadyCheck" => {
            println!("🎯 Ready check phase detected!");
        }
        "InProgress" => {
            println!("🎮 Game in progress");
        }
        "WaitingForStats" => {
            println!("📊 Waiting for post-game stats");
        }
        "PreEndOfGame" => {
            println!("🏁 Game ending");
        }
        "EndOfGame" => {
            println!("🎯 Game ended, returning to lobby");
        }
        "None" => {
            println!("⚪ No active game state");
        }
        _ => {
            println!("🔄 Client State: {}", client_state);
        }
    }
}

async fn handle_champ_select_start(
    client: &LcuClient,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let team = get_lobby_info(client).await?;
    
    if team.participants.is_empty() {
        println!("⚠️ No participants found in champion select.");
        return Ok(());
    }

    let region_info = client.get_region_info().await?;
    println!("🌍 Region: {}", region_info.web_region);

    if config.auto_open {
        let region = match region_info.web_region.as_str() {
            "SG2" => "SG",
            _ => &region_info.web_region,
        };

        display_champ_select(&team, region);
    }

    Ok(())
}