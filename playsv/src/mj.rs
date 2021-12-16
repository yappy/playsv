use super::mjsys;
use std::collections::VecDeque;
use std::sync::RwLock;
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};

/*
Game State Machine

AP : Active Player
NAP: Non Active Player

1. After draw
* AP can:
** Trash one -> 2
** Kan (open/blind) -> 1
** Tsumo -> 3
* Any NAP can do nothing.

2. After trash
* NAP can:
** Chi -> 1
** Pon -> 1
** Kan -> 1
** Ron -> 3
** No Reaction -> 1

3. Result
* Wait for response from all players.
* Go to next or finish game.

*/

// each game is protected by indivisual rwlock
pub struct Game {
    state: RwLock<GameState>,
}

#[derive(Debug)]
struct GameState {
    // constant
    player_count: u32,
    round_max: u32,
    // variable
    round: u32,
    points: [u32; 4],
    yama: VecDeque<u8>,
    hands: [Vec<u8>; 4],
}

// board view from each player
#[derive(Debug, Serialize, Deserialize)]
struct View {
    hand: Vec<u8>,
    // != 0 if you have drawn this hai just now
    draw: u8,
}

impl Game {
    pub fn new() -> Result<Game, String> {
        let mut state = GameState::new();
        // TODO: pass rule config
        match state.init() {
            // After this, it is necessary to take a lock for GameState access
            Ok(()) => Ok(Game {state: RwLock::new(state)}),
            Err(msg) => Err(msg),
        }
    }

    #[allow(dead_code)]
    pub fn get_view(&self, player: u32) -> Result<(), String> {
        let state = self.state.read().unwrap();
        if player >= state.player_count {
            return Err(format!("Invalid player: {}", player));
        }

        // TODO
        Ok(())
    }

    #[allow(dead_code)]
    pub fn action(&mut self) {
        let mut _state = self.state.write().unwrap();
        // ...
    }
}

impl GameState {
    fn new() -> GameState {
        GameState {
            player_count: 0,
            round_max: 0,
            round: 0,
            points: Default::default(),
            yama: VecDeque::new(),
            hands: Default::default(),
        }
    }

    fn init(&mut self) -> Result<(), String> {
        // TODO: receive rule config and set
        self.player_count = 2;
        let player_size = self.player_count as usize;
        self.round_max = 4;
        self.round = 1;
        for (i, p) in self.points.iter_mut().enumerate() {
            *p = if i < player_size { 25000 } else { 0 };
        }
        self.yama.clear();
        for hand in self.hands.iter_mut() {
            hand.clear();
        }

        // init as Round 1 start state
        self.next_round(1);

        Ok(())
    }

    fn next_round(&mut self, round: u32) {
        self.round = round;
        // Create yama
        {
            let mut yama_tmp: Vec<u8> = vec![];
            // man, pin, so: 0, 1, 2
            for kind in 0..3 {
                // 1-9
                for num in 1..=9 {
                    yama_tmp.push(mjsys::encode(kind, num, 0));
                }
            }
            // zu: 3
            for num in 1..=7 {
                yama_tmp.push(mjsys::encode(3, num, 0));
            }
            // thread_local cryptographically secure PRNG
            let mut rng = rand::thread_rng();
            yama_tmp.shuffle(&mut rng);

            self.yama = yama_tmp.into();
        }
        // TODO: haipai
    }
}
