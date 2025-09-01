use crate::{lobby::Lobby, region::RegionInfo, summoner::Summoner};

pub async fn send_analytics_event(
    _team: &Lobby,
    _summoner: &Summoner,
    _region_info: &RegionInfo,
) {
    // Analytics functionality removed for standalone console app
    // This is a placeholder to maintain compatibility
    println!("Analytics event would be sent here (disabled in console version)");
}