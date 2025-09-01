use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use urlencoding::encode;

static SLEEP_BETWEEN_POLLS: Lazy<Duration> = Lazy::new(|| Duration::from_secs(2));

#[derive(Debug, Error)]
enum LcuError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Clone, Debug)]
struct LcuAuthInfo {
    port: u16,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegionInfo {
    locale: Option<String>,
    region: Option<String>,
    web_language: Option<String>,
    web_region: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Lobby {
    participants: Vec<Participant>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Participant {
    cid: Option<String>,
    game_name: String,
    game_tag: String,
    muted: Option<bool>,
    name: Option<String>,
    pid: Option<String>,
    puuid: Option<String>,
    region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectSession {
    game_id: u64,
}

#[tokio::main]
async fn main() {
    println!("opgg-console starting. Auto-accept and auto-open op.gg are enabled.");

    // Wait for lockfile and build client
    let mut client = build_client();
    let mut auth = loop {
        match read_lockfile().and_then(|info| Ok(info)) {
            Ok(info) => break info,
            Err(_) => {
                println!("Waiting for League Client lockfile...");
                thread::sleep(*SLEEP_BETWEEN_POLLS);
            }
        }
    };

    let mut last_opened_game_id: Option<u64> = None;

    loop {
        // If any step fails, try to re-read lockfile and rebuild client
        match get_gameflow_phase(&client, &auth).await {
            Ok(phase) => {
                match phase.as_str() {
                    "ReadyCheck" => {
                        if let Err(err) = accept_ready_check(&client, &auth).await {
                            eprintln!("Failed to accept ready check: {}", err);
                        } else {
                            println!("Accepted ready check.");
                        }
                    }
                    "ChampSelect" => {
                        if let Ok(session) = get_champ_select_session(&client, &auth).await {
                            let game_id = session.game_id;
                            if last_opened_game_id != Some(game_id) {
                                if let Err(err) = open_opgg_multi_for_team(&client, &auth).await {
                                    eprintln!("Failed to open op.gg multi: {}", err);
                                } else {
                                    println!("Opened op.gg multi for current lobby.");
                                    last_opened_game_id = Some(game_id);
                                }
                            }
                        } else {
                            // Try without session info (best-effort)
                            if last_opened_game_id.is_none() {
                                if let Err(err) = open_opgg_multi_for_team(&client, &auth).await {
                                    eprintln!("Failed to open op.gg multi: {}", err);
                                } else {
                                    println!("Opened op.gg multi for current lobby.");
                                }
                            }
                        }
                    }
                    _ => {
                        // Reset our per-game flag when outside ChampSelect
                        last_opened_game_id = None;
                    }
                }
            }
            Err(err) => {
                eprintln!("Error talking to LCU: {}", err);
                // Re-read lockfile and rebuild client; wait a bit
                match read_lockfile() {
                    Ok(new_auth) => {
                        auth = new_auth;
                        client = build_client();
                    }
                    Err(_) => {
                        println!("League Client not detected. Retrying...");
                    }
                }
            }
        }

        thread::sleep(*SLEEP_BETWEEN_POLLS);
    }
}

fn build_client() -> Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
}

fn read_lockfile() -> Result<LcuAuthInfo, LcuError> {
    // Allow override via env var LEAGUE_LOCKFILE
    if let Ok(custom) = std::env::var("LEAGUE_LOCKFILE") {
        let info = parse_lockfile(&PathBuf::from(custom))?;
        return Ok(info);
    }

    let candidates = default_lockfile_candidates();
    for path in candidates {
        if path.exists() {
            if let Ok(info) = parse_lockfile(&path) {
                return Ok(info);
            }
        }
    }
    Err(LcuError::Parse("Lockfile not found".into()))
}

fn default_lockfile_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if cfg!(target_os = "windows") {
        paths.push(PathBuf::from(r"C:\\Riot Games\\League of Legends\\lockfile"));
    } else if cfg!(target_os = "macos") {
        paths.push(PathBuf::from(
            "/Applications/League of Legends.app/Contents/LoL/lockfile",
        ));
    } else {
        // Common Lutris path on Linux
        paths.push(PathBuf::from(
            std::env::var("HOME").map(|h| format!("{}/.local/share/lutris/runners/leagueoflegends/lockfile", h)).unwrap_or_else(|_| "/tmp/lockfile".to_string()),
        ));
    }

    paths
}

fn parse_lockfile(path: &Path) -> Result<LcuAuthInfo, LcuError> {
    let content = fs::read_to_string(path)?;
    // Format: name:pid:port:password:protocol
    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() < 5 {
        return Err(LcuError::Parse("Invalid lockfile format".into()));
    }
    let port: u16 = parts[2]
        .parse()
        .map_err(|e| LcuError::Parse(format!("Invalid port: {}", e)))?;
    let password = parts[3].to_string();
    Ok(LcuAuthInfo { port, password })
}

async fn get_gameflow_phase(client: &Client, auth: &LcuAuthInfo) -> Result<String, LcuError> {
    let url = format!(
        "https://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase",
        auth.port
    );
    let text = client
        .get(url)
        .basic_auth("riot", Some(&auth.password))
        .send()
        .await?
        .text()
        .await?;
    Ok(text.trim_matches('"').to_string())
}

async fn accept_ready_check(client: &Client, auth: &LcuAuthInfo) -> Result<(), LcuError> {
    let url = format!(
        "https://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept",
        auth.port
    );
    client
        .post(url)
        .basic_auth("riot", Some(&auth.password))
        .json(&serde_json::json!({}))
        .send()
        .await?;
    Ok(())
}

async fn get_champ_select_session(
    client: &Client,
    auth: &LcuAuthInfo,
) -> Result<ChampSelectSession, LcuError> {
    let url = format!(
        "https://127.0.0.1:{}/lol-champ-select/v1/session",
        auth.port
    );
    let session = client
        .get(url)
        .basic_auth("riot", Some(&auth.password))
        .send()
        .await?
        .json::<ChampSelectSession>()
        .await?;
    Ok(session)
}

async fn get_chat_participants(client: &Client, auth: &LcuAuthInfo) -> Result<Lobby, LcuError> {
    let url = format!("https://127.0.0.1:{}/chat/v5/participants", auth.port);
    let lobby = client
        .get(url)
        .basic_auth("riot", Some(&auth.password))
        .send()
        .await?
        .json::<Lobby>()
        .await?;
    Ok(lobby)
}

async fn get_region_info(client: &Client, auth: &LcuAuthInfo) -> Result<RegionInfo, LcuError> {
    let url = format!(
        "https://127.0.0.1:{}/riotclient/region-locale",
        auth.port
    );
    let info = client
        .get(url)
        .basic_auth("riot", Some(&auth.password))
        .send()
        .await?
        .json::<RegionInfo>()
        .await?;
    Ok(info)
}

async fn open_opgg_multi_for_team(client: &Client, auth: &LcuAuthInfo) -> Result<(), LcuError> {
    let team = get_chat_participants(client, auth).await?;
    let region_info = get_region_info(client, auth).await?;

    let region = match region_info.web_region.as_str() {
        "SG2" => "SG".to_string(),
        other => other.to_string(),
    };

    let link = create_opgg_link(&team.participants, &region);
    if let Err(err) = open::that(&link) {
        return Err(LcuError::Parse(format!(
            "Failed to open browser for {}: {}",
            link, err
        )));
    }
    Ok(())
}

fn create_opgg_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://www.op.gg/multisearch/{}?summoners=", region);
    let mut link_path = String::new();
    for summoner in summoners {
        // Only include those from champ-select chat if cid present
        if let Some(cid) = &summoner.cid {
            if !cid.contains("champ-select") {
                continue;
            }
        }

        let full_tag = format!("{}#{}", summoner.game_name, summoner.game_tag);
        link_path.push_str(&full_tag);
        link_path.push(',');
    }
    if link_path.ends_with(',') {
        link_path.pop();
    }

    let encoded_path = encode(&link_path);
    format!("{}{}", base_url, encoded_path)
}

