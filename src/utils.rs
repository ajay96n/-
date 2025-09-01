use crate::lobby::{Lobby, Participant};
use urlencoding::encode;

pub fn create_opgg_link(summoners: &Vec<Participant>, region: &str) -> String {
    let base_url = format!("https://www.op.gg/multisearch/{}?summoners=", region);
    let mut link_path = String::new();
    for summoner in summoners {
        let full_tag = format!("{}#{}", summoner.game_name, summoner.game_tag);
        link_path.push_str(&full_tag);
        link_path.push(',');
    }
    link_path.pop();

    let encoded_path = encode(&link_path);
    format!("{}{}", base_url, encoded_path)
}

pub fn display_champ_select(lobby: &Lobby, region: &str) {
    if lobby.participants.is_empty() {
        println!("No participants found in champion select.");
        return;
    }

    let mut team_string = String::new();
    for summoner in lobby.participants.iter() {
        let participant = format!(
            "{}#{} ({})",
            summoner.game_name, summoner.game_tag, summoner.name
        );
        team_string.push_str(&participant);
        if summoner.name != lobby.participants.last().unwrap().name {
            team_string.push_str(", ");
        }
    }

    println!("Team: {}", team_string);
    let link = create_opgg_link(&lobby.participants, region);
    println!("Opening OP.GG Multi: {}", link);

    match open::that(&link) {
        Ok(_) => println!("Successfully opened OP.GG Multi in browser"),
        Err(e) => println!("Failed to open link in browser: {}", e),
    }
}