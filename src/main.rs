mod champ_select;
mod config;
mod lobby;
mod region;
mod utils;

use crate::champ_select::ChampSelectSession;
use crate::config::Config;
use crate::lobby::{get_lobby_info, Lobby};
use crate::region::RegionInfo;
use crate::utils::display_champ_select;
use futures_util::StreamExt;
use shaco::model::ws::LcuEvent;
use shaco::rest::RESTClient;
use shaco::utils::process_info;
use shaco::ws::LcuWebsocketClient;
use shaco::{model::ws::LcuSubscriptionType::JsonApiEvent, rest::LCUClientInfo};
use std::time::Duration;
use tokio::time::sleep;

struct AppState {
    config: Config,
    lcu_info: Option<LCUClientInfo>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            config: Config::default(), // Hardcoded config with auto_open=true, auto_accept=true, multi_provider=opgg
            lcu_info: None,
        }
    }
}

#[tokio::main]
async fn main() {
    println!("🎮 League of Legends Auto Opener Console Application");
    println!("📋 Configuration:");
    println!("   • Auto Open Multi: ENABLED (OP.GG)");
    println!("   • Auto Accept: ENABLED");
    println!("   • Multi Provider: OP.GG (FIXED)");
    println!("🔄 Starting application...\n");

    let mut app_state = AppState::new();
    let mut connected = false;

    loop {
        let args = process_info::get_league_process_args();
        if args.is_none() {
            if connected {
                println!("❌ League Client disconnected. Waiting for reconnection...");
                connected = false;
            } else {
                println!("⏳ Waiting for League Client to open...");
            }
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        let args = args.unwrap();
        let lcu_info = match process_info::get_auth_info(args) {
            Ok(info) => info,
            Err(e) => {
                println!("❌ Failed to get LCU auth info: {}", e);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        app_state.lcu_info = Some(lcu_info.clone());

        let app_client = match RESTClient::new(lcu_info.clone(), false) {
            Ok(client) => client,
            Err(e) => {
                println!("❌ Failed to create app client: {}", e);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let remoting_client = match RESTClient::new(lcu_info.clone(), true) {
            Ok(client) => client,
            Err(e) => {
                println!("❌ Failed to create remoting client: {}", e);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        connected = true;
        println!("✅ Connected to League Client!");

        // Connect to websocket with retry logic
        let mut ws = match LcuWebsocketClient::connect().await {
            Ok(ws) => ws,
            Err(_) => {
                println!("⏳ Waiting for websocket connection...");
                let mut attempts = 0;
                loop {
                    sleep(Duration::from_secs(3)).await;
                    if attempts > 5 {
                        println!("❌ Failed to connect to League Client websocket after 5 attempts!");
                        break;
                    }

                    attempts += 1;
                    match LcuWebsocketClient::connect().await {
                        Ok(ws) => break ws,
                        Err(_) => {
                            println!("🔄 Websocket connection attempt {} failed, retrying...", attempts);
                            continue;
                        }
                    }
                }
            }
        };

        // Subscribe to game flow events
        if let Err(e) = ws.subscribe(JsonApiEvent("/lol-gameflow/v1/gameflow-phase".to_string())).await {
            println!("❌ Failed to subscribe to gameflow events: {}", e);
            continue;
        }

        // Subscribe to champion select events
        if let Err(e) = ws.subscribe(JsonApiEvent("/lol-champ-select/v1/session".to_string())).await {
            println!("❌ Failed to subscribe to champion select events: {}", e);
            continue;
        }

        println!("🎯 Subscribed to League Client events. Monitoring for champion select...\n");

        // Check initial game state
        if let Ok(state) = get_gameflow_state(&remoting_client).await {
            handle_client_state(state, &app_state, &remoting_client, &app_client).await;
        }

        // Main event loop
        while let Some(msg) = ws.next().await {
            if handle_ws_message(msg, &app_state, &remoting_client, &app_client).await.is_err() {
                break; // Connection lost, restart
            }
        }

        println!("🔄 Connection lost, attempting to reconnect...");
        connected = false;
    }
}

async fn get_gameflow_state(remoting_client: &RESTClient) -> Result<String, Box<dyn std::error::Error>> {
    let gameflow_state = remoting_client
        .get("/lol-gameflow/v1/gameflow-phase".to_string())
        .await?
        .to_string();

    let cleaned_state = gameflow_state.replace('\"', "");
    Ok(cleaned_state)
}

async fn handle_client_state(
    client_state: String,
    app_state: &AppState,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) {
    match client_state.as_str() {
        "ChampSelect" => {
            println!("🎯 Champion Select detected!");
            
            // Wait a bit for the champion select to fully load
            sleep(Duration::from_secs(5)).await;
            
            if let Err(e) = handle_champ_select_start(app_client, remoting_client, &app_state.config).await {
                println!("❌ Error handling champion select: {}", e);
            }
        }
        "ReadyCheck" => {
            if app_state.config.auto_accept {
                println!("🎯 Ready check detected! Auto-accepting in {}ms...", app_state.config.accept_delay);
                
                sleep(Duration::from_millis(
                    (app_state.config.accept_delay as u64).saturating_sub(1000),
                )).await;
                
                match remoting_client
                    .post(
                        "/lol-matchmaking/v1/ready-check/accept".to_string(),
                        serde_json::json!({}),
                    )
                    .await
                {
                    Ok(_) => println!("✅ Ready check accepted!"),
                    Err(e) => println!("❌ Failed to accept ready check: {}", e),
                }
            }
        }
        "Lobby" => {
            println!("🏠 In lobby, waiting for queue...");
        }
        "Matchmaking" => {
            println!("🔍 Searching for match...");
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
    app_client: &RESTClient,
    remoting_client: &RESTClient,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let team = get_lobby_info(app_client).await?;
    
    if team.participants.is_empty() {
        println!("⚠️ No participants found in champion select.");
        return Ok(());
    }

    let region_info: RegionInfo = serde_json::from_value(
        app_client
            .get("/riotclient/region-locale".to_string())
            .await?,
    )?;

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

async fn handle_ws_message(
    msg: LcuEvent,
    app_state: &AppState,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg_type = msg.subscription_type.to_string();

    match msg_type.as_str() {
        "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase" => {
            let client_state = msg.data.to_string().replace('\"', "");
            handle_client_state(client_state, app_state, remoting_client, app_client).await;
        }
        "OnJsonApiEvent_lol-champ-select_v1_session" => {
            // Handle champion select session updates if needed
            if let Ok(_champ_select) = serde_json::from_value::<ChampSelectSession>(msg.data.clone()) {
                // Champion select session updated - could add additional logic here
            }
        }
        _ => {
            // Ignore other message types
        }
    }

    Ok(())
}