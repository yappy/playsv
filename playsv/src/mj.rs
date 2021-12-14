use super::mjsys;
use std::collections::VecDeque;
use std::sync::RwLock;
use rand::seq::SliceRandom;

// each game is protected by indivisual rwlock
pub struct Game {
    state: RwLock<GameState>,
}

#[derive(Debug)]
struct GameState {
    member_count: u32,
    yama: VecDeque<u8>,
    hand: Vec<Vec<u8>>,
}

impl Game {
    pub fn new() -> Game {
        let state = GameState {
            member_count: 4,
            yama: VecDeque::new(),
            hand: vec![]
        };

        Game {
            state: RwLock::new(state)
        }
    }

    pub fn init(&self) {
        let mut state = self.state.write().unwrap();
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

            state.yama = yama_tmp.into()
        }
    }
}
