/*
Encoding:
[5:4]: (2 bit) kind (man, pin, so, zu)
[3:0]: (4 bit) 1..9 number (1..7 for zu-hai)

*/

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
