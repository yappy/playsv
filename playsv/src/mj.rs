use super::mjsys;
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

// main game status control data
#[derive(Debug)]
struct GameState {
    // common view to all players as is
    common: CommonState,
    // hidden or player-dependent view
    internal: InternalState,
}

// publish to each player as json
#[derive(Debug, Serialize, Deserialize)]
struct LocalView {
    common: CommonState,
    local: LocalState,
}

// the same view from all players
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct CommonState {
    // constant
    player_count: u32,
    round_max: u32,
    // active player index and phase
    turn: u32,
    phase: GamePhase,
    // wind 0..round_max, parent 0..player_count, hon
    wind: u32,
    parent: u32,
    hon: u32,
    // TODO
    // dora
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum GamePhase {
    WaitAction,
    WaitReaction,
    ShowResult,
}

// player-dependent data (managed by system)
#[derive(Debug, Default)]
struct InternalState {
    points: Vec<i32>,
    yama: Vec<i32>,
    // wang pai
    yama2: Vec<i32>,
    hands: Vec<Vec<i32>>,
}

// player-dependent data view
#[derive(Debug, Default, Serialize, Deserialize)]
struct LocalState {
    points: [i32; 4],
    hands: [Vec<i32>; 4],
}

impl Game {
    pub fn new() -> Result<Game, String> {
        let mut state = GameState::new();
        // TODO: pass rule config
        match state.init() {
            // After this, it is necessary to take a lock for GameState access
            Ok(()) => Ok(Game { state: RwLock::new(state) }),
            Err(msg) => Err(msg),
        }
    }

    pub fn get_view(&self, player: u32) -> Result<String, String> {
        // read lock and (common, internal) <- state
        let GameState { common, internal } = &*self.state.read().unwrap();
        if player >= common.player_count {
            return Err("Invalid player: {}".to_string());
        }

        // result struct for json output
        // copy common field
        let mut result = LocalView { common: *common, local: Default::default() };
        // covert internal -> local view
        // TODO
        for i in 0..common.player_count {
            // i = global player index
            // p = local player index
            let p = (player + i) % common.player_count;

            let i = i as usize;
            let p = p as usize;

            // TODO covert hand
        }

        // return as json string
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
            common: CommonState {
                player_count: 0,
                round_max: 0,
                turn: 0,
                phase: GamePhase::WaitAction,
                wind: 0,
                parent: 0,
                hon: 0,
            },
            internal: Default::default(),
        }
    }

    fn check(&self) {
        let GameState { common, internal } = self;
        assert!(2 >= common.player_count && common.player_count <= 4);
        assert!(common.round_max <= 4);
        assert!(common.wind < 4);
        assert!(common.parent < common.player_count);
        assert!(common.turn < common.player_count);
    }

    fn init(&mut self) -> Result<(), String> {
        let (common, internal) = (&mut self.common, &mut self.internal);

        // TODO: receive rule config and set
        common.player_count = 2;
        let player_size = common.player_count as usize;
        common.round_max = 4;
        common.turn = 0;
        common.wind = 0;
        common.parent = 0;
        common.hon = 0;
        for (i, p) in internal.points.iter_mut().enumerate() {
            *p = if i < player_size { 25000 } else { 0 };
        }
        internal.yama.clear();
        for _ in 0..common.player_count {
            internal.hands.push(vec![]);
        }

        // init as tong 1 kyoku 0 hon start
        self.next_round(0, 0, 0);

        self.check();

        Ok(())
    }

    fn next_round(&mut self, wind: u32, parent: u32, hon: u32) {
        let (common, internal) = (&mut self.common, &mut self.internal);

        common.wind = wind;
        common.parent = parent;
        common.hon = hon;
        // the first turn player == parent == kyoku number
        common.turn = parent;
        // Create yama
        {
            let mut yama_tmp: Vec<i32> = vec![];
            // man, pin, so: 0, 1, 2
            for kind in 0..3 {
                // 1-9
                for num in 1..=9 {
                    yama_tmp.push(mjsys::encode(kind, num, 0) as i32);
                }
            }
            // zu: 3
            for num in 1..=7 {
                yama_tmp.push(mjsys::encode(3, num, 0) as i32);
            }
            // thread_local cryptographically secure PRNG
            let mut rng = rand::thread_rng();
            yama_tmp.shuffle(&mut rng);

            internal.yama = yama_tmp.into();
        }
        // haipai
        {
            // 4 times
            for count in 0..4 {
                // parent-origin, for each player
                for i in 0..common.player_count {
                    let player = (parent + i) % common.player_count;
                    // take 4, 4, 4, 1
                    let take_num = if count == 3 { 1 } else { 4 };
                    for _ in 0..take_num {
                        let hai = internal.yama.pop().unwrap();
                        internal.hands[player as usize].push(hai);
                    }
                }
            }
            for i in 0..common.player_count {
                assert!(internal.hands[i as usize].len() == 13);
            }
        }
        self.check();
    }
}
