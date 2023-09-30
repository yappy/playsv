use serde::{Deserialize, Serialize};

// publish to each player
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalView {
    pub common: CommonState,
    pub local: LocalState,
}

// the same view from all players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonState {
    // constant
    pub player_count: u32,
    pub round_max: u32,
    // active player index and phase
    pub turn: u32,
    pub phase: GamePhase,
    // wind 0..round_max, parent 0..player_count, hon
    pub wind: u32,
    pub parent: u32,
    pub hon: u32,
    // TODO
    // dora
}

// player-dependent data view
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocalState {
    pub points: [i32; 4],
    pub hands: [Vec<i32>; 4],
    pub hands_str: [Vec<String>; 4],
    pub draws: [i32; 4],
    pub draws_str: [String; 4],
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GamePhase {
    WaitAction,
    WaitReaction,
    ShowResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    // Active player
    Discard(i32),
    Tsumo,
    // TODO: param
    BlindKan,
    SmallKan,

    // Non-active player
    Skip,
    Ron,
    // TODO: param
    Chi,
    Pon,
    BigKan,
}
