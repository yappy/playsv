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
    // wind 0..round_max, parent 0..player_count, hon
    wind: u32,
    parent: u32,
    hon: u32,
    points: [u32; 4],
    yama: VecDeque<u8>,
    turn: u32,
    hands: [Vec<u8>; 4],
}

// board view from each player
#[derive(Debug, Serialize, Deserialize)]
struct View {
    hand: [Vec<i32>; 4],
    // != 0 if you have drawn this hai just now
    draw: [i32; 4],
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

    pub fn get_view(&self, player: u32) -> Result<String, String> {
        let state = self.state.read().unwrap();
        if player >= state.player_count {
            return Err("Invalid player: {}".to_string());
        }

        // TODO
        let result = View{hand: Default::default(), draw: [0; 4]};
        let result = serde_json::to_string(&result).unwrap();

        Ok(result)
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
            wind: 0,
            parent: 0,
            hon: 0,
            points: Default::default(),
            yama: VecDeque::new(),
            turn: 0,
            hands: Default::default(),
        }
    }

    fn check(&self) {
        assert!(2 >= self.player_count && self.player_count <= 4);
        assert!(self.round_max <= 4);
        assert!(self.wind < 4);
        assert!(self.parent < self.player_count);
        assert!(self.turn < self.player_count);
    }

    fn init(&mut self) -> Result<(), String> {
        // TODO: receive rule config and set
        self.player_count = 2;
        let player_size = self.player_count as usize;
        self.round_max = 4;
        self.wind = 0;
        self.parent = 0;
        self.hon = 0;
        for (i, p) in self.points.iter_mut().enumerate() {
            *p = if i < player_size { 25000 } else { 0 };
        }
        self.yama.clear();
        self.turn = 0;
        for hand in self.hands.iter_mut() {
            hand.clear();
        }

        // init as tong 1 kyoku 0 hon start
        self.next_round(0, 0, 0);

        self.check();

        Ok(())
    }

    fn next_round(&mut self, wind: u32, parent: u32, hon: u32) {
        self.wind = wind;
        self.parent = parent;
        self.hon = hon;
        // the first turn player == parent == kyoku number
        self.turn = parent;
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
        // haipai
        {
            // 4 times
            for count in 0..4 {
                // parent-origin, for each player
                for i in 0..self.player_count {
                    let player = (parent + i) % self.player_count;
                    // take 4, 4, 4, 1
                    let take_num = if count == 3 { 1 } else { 4 };
                    for _ in 0..take_num {
                        let hai = self.yama.pop_back().unwrap();
                        self.hands[player as usize].push(hai);
                    }
                }
            }
            for i in 0..self.player_count {
                assert!(self.hands[i as usize].len() == 13);
            }
        }
        self.check();
    }
}
