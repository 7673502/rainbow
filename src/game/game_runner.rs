use std::collections::HashMap;

use crate::agents::Agent;
use crate::game::game_state::GameState;

pub struct GameRunner {
    state: GameState,
    agents: HashMap<u8, Box<dyn Agent>>,
}

impl GameRunner {
    pub fn new(
        participants: impl IntoIterator<Item = (u8, Box<dyn Agent>)>,
        seed: Option<u64>,
    ) -> Self {
        let mut player_uids = Vec::new();
        let mut agents = HashMap::new();

        for (uid, agent) in participants {
            player_uids.push(uid);
            agents.insert(uid, agent);
        }

        GameRunner {
            state: GameState::new(player_uids, seed),
            agents: agents.into_iter().collect(),
        }
    }

    pub fn run_game(&mut self) -> &GameState {
        while !self.run_iteration() {}

        &self.state
    }

    fn run_iteration(&mut self) -> bool {
        let current_player_uid = self.state.get_current_player_uid();
        let current_agent = &self.agents[&current_player_uid];

        let valid_actions = self.state.get_legal_actions(current_player_uid);
        let scrubbed_state = self.state.scrub_state(current_player_uid);

        let choice_index = current_agent.choose_action(scrubbed_state, &valid_actions);
        let chosen_combo = match valid_actions.get(choice_index) {
            Some(combo) => combo,
            None => panic!(
                "agent with uid {} returned invalid action index {}",
                current_player_uid, choice_index
            ),
        };
        self.state.apply_action(*chosen_combo);

        self.state.get_is_game_over()
    }
}
