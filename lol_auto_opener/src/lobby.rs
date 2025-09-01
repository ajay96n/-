use serde::{Deserialize, Serialize};
use crate::lcu_client::LcuClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct Participant {
    pub cid: String,
    pub game_name: String,
    pub game_tag: String,
    pub muted: bool,
    pub name: String,
    pub pid: String,
    pub puuid: String,
    pub region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby {
    pub participants: Vec<Participant>,
}

pub async fn get_lobby_info(client: &LcuClient) -> Result<Lobby, Box<dyn std::error::Error>> {
    let mut lobby = client.get_lobby_participants().await?;
    
    // Filter out all cids that contain champ-select
    lobby.participants = lobby
        .participants
        .into_iter()
        .filter(|p| p.cid.contains("champ-select"))
        .collect::<Vec<Participant>>();

    Ok(lobby)
}