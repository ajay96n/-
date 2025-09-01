use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub allow_battle_boost: bool,
    pub allow_duplicate_picks: bool,
    pub allow_locked_events: bool,
    pub allow_rerolling: bool,
    pub allow_skin_selection: bool,
    pub bench_enabled: bool,
    pub boostable_skin_count: i64,
    pub counter: i64,
    pub game_id: u64,
    pub has_simultaneous_bans: bool,
    pub has_simultaneous_picks: bool,
    pub is_custom_game: bool,
    pub is_spectating: bool,
    pub local_player_cell_id: i64,
    pub locked_event_index: i64,
    pub recovery_counter: i64,
    pub rerolls_remaining: i64,
    pub skip_champion_select: bool,
    pub timer: Timer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub adjusted_time_left_in_phase: u64,
    pub internal_now_in_epoch_ms: u64,
    pub is_infinite: bool,
    pub phase: String,
    pub total_time_in_phase: i64,
}