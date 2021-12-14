use super::mjsys;
use std::collections::VecDeque;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct GameState {
    member_count: u32,
    yama: VecDeque<u8>,
    hand: Vec<Vec<u8>>,
}

impl GameState {
    pub fn new() -> GameState {
        let mut obj = GameState {
            member_count: 4,
            yama: VecDeque::new(),
            hand: vec![]
        };

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

            obj.yama = yama_tmp.into()
        }

        obj
    }
}
