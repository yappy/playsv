
struct GameState {
    member_count: u32,
    yama: Vec<u8>,
    hand: Vec<Vec<u8>>,
}

impl GameState {
    fn new() -> GameState {
        GameState { member_count: 4, yama: vec![], hand: vec![] }
    }
}
