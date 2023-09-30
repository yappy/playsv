use crate::jsif;
use crate::mjsys;
use rand::seq::SliceRandom;
use std::sync::RwLock;

/*
Game State Machine

AP : Active Player
NAP: Non Active Player

1. After draw
* AP can:
** Trash one -> 2
** Kan (open/blind) -> 1
** Tsumo -> 3
* Any NAPs can do nothing.

2. After trash
* NAPs can:
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
pub struct Game(RwLock<GameState>);

// main game status control data
#[derive(Debug)]
struct GameState {
    // common view to all players as is
    common: jsif::CommonState,
    // hidden or player-dependent view
    internal: InternalState,
}

// player-dependent data (managed by system)
#[derive(Debug, Default)]
struct InternalState {
    yama: Vec<i32>,
    // wang pai
    yama2: Vec<i32>,
    points: Vec<i32>,
    hands: Vec<Vec<i32>>,
    draws: Vec<Option<i32>>,
    actions: Vec<Vec<jsif::Action>>,
}

impl Game {
    pub fn new() -> Result<Game, String> {
        let mut state = GameState::new();
        // TODO: pass rule config
        match state.init() {
            // After this, it is necessary to take a lock for GameState access
            Ok(()) => Ok(Game(RwLock::new(state))),
            Err(msg) => Err(msg),
        }
    }

    pub fn get_view(&self, player: u32) -> Result<String, String> {
        // read lock and (common, internal) <- state
        let GameState { common, internal } = &*self.0.read().unwrap();
        if player >= common.player_count {
            return Err("Invalid player: {}".to_string());
        }

        // result struct for json output
        let result = jsif::LocalView {
            // copy common field
            common: common.clone(),
            // convert from internal
            local: Self::convert_to_local_state(common, internal, player),
        };

        // return as json string
        let result = serde_json::to_string(&result).unwrap();
        Ok(result)
    }

    fn convert_to_local_state(
        common: &jsif::CommonState,
        internal: &InternalState,
        player: u32,
    ) -> jsif::LocalState {
        let mut local: jsif::LocalState = Default::default();

        for i in 0..4 {
            // i = global player index
            // p = local player index
            let p = (player + i) % 4;

            let ius = i as usize;
            let pus = p as usize;

            if p < common.player_count {
                // hand
                local.points[ius] = internal.points[pus];
                local.hands[ius] = internal.hands[pus].clone();
                for hai in &*local.hands[ius] {
                    let code = *hai as u16;
                    local.hands_str[ius].push(mjsys::human_readable_string(code));
                }
                // draw
                local.draws[ius] = match internal.draws[pus] {
                    Some(hai) => hai,
                    None => -1,
                };
                local.draws_str[ius] = match internal.draws[pus] {
                    Some(hai) => mjsys::human_readable_string(hai as u16),
                    None => "".to_string(),
                };
                // action
                if pus == 0 {
                    local.actions = internal.actions[ius].clone();
                }
            } else {
                // empty seat
                local.points[ius] = i32::MIN;
                // local.hands[i] = [];
                local.draws[ius] = -1;
            }
        }

        local
    }

    #[allow(dead_code)]
    pub fn action(&mut self) {
        let mut _state = self.0.write().unwrap();
        // ...
    }
}

impl GameState {
    fn new() -> GameState {
        GameState {
            common: jsif::CommonState {
                player_count: 0,
                round_max: 0,
                turn: 0,
                phase: jsif::GamePhase::WaitAction,
                wind: 0,
                parent: 0,
                hon: 0,
            },
            internal: Default::default(),
        }
    }

    fn check(&self) {
        let (common, _internal) = (&self.common, &self.internal);

        assert!(2 <= common.player_count && common.player_count <= 4);
        assert!(common.round_max <= 4);
        assert!(common.wind < 4);
        assert!(common.parent < common.player_count);
        assert!(common.turn < common.player_count);
    }

    fn init(&mut self) -> Result<(), String> {
        let (common, internal) = (&mut self.common, &mut self.internal);

        // TODO: receive rule config and set
        common.player_count = 4;
        common.round_max = 4;
        common.turn = 0;
        common.wind = 0;
        common.parent = 0;
        common.hon = 0;

        internal.yama.clear();
        internal.yama2.clear();
        for _ in 0..common.player_count {
            internal.points.push(25000);
            internal.hands.push(vec![]);
            internal.draws.push(None);
            internal.actions.push(vec![]);
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
            for _ in 0..4 {
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
            }
            // thread_local cryptographically secure PRNG
            let mut rng = rand::thread_rng();
            yama_tmp.shuffle(&mut rng);

            internal.yama = yama_tmp;
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
                let i = i as usize;
                assert!(internal.hands[i].len() == 13);
                internal.hands[i].sort_unstable();
            }
            // parent draw
            self.draw();
        }
        self.check();
    }

    // Turn player draws a hai
    fn draw(&mut self) {
        let (common, internal) = (&mut self.common, &mut self.internal);

        // all player must not have draw hai
        // yama.len() must be > 0
        for p in 0..common.player_count as usize {
            assert!(internal.draws[p] == None);
        }
        // draw
        internal.draws[common.turn as usize] = Some(internal.yama.pop().unwrap());

        // go to new state
        common.phase = jsif::GamePhase::WaitAction;
        for alist in &mut internal.actions {
            alist.clear();
        }
        // create action list
        let ap_actions = &mut internal.actions[common.turn as usize];
        let ap_hand = &internal.hands[common.turn as usize];
        for (i, _hai) in ap_hand.iter().enumerate() {
            ap_actions.push(jsif::Action::Discard(i as i32));
        }
    }
}
