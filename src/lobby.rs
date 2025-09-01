use serde::{Deserialize, Serialize};
use crate::lcu_client::LCUClient;

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

pub async fn get_lobby_info(app_client: &LCUClient) -> Lobby {
    let response = app_client
        .get("/chat/v5/participants")
        .await
        .unwrap();
    
    let team: Lobby = response.json().await.unwrap();

    // filter out all cids that contain champ-select
    let team_participants = team
        .participants
        .into_iter()
        .filter(|p| p.cid.contains("champ-select"))
        .collect::<Vec<Participant>>();

    let team = Lobby {
        participants: team_participants,
    };

    team
}