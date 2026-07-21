use std::collections::HashMap;

use crate::agents::Agent;
use crate::game::game_state::GameState;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::random_agent::RandomAgent;
    use proptest::prelude::*;

    const MAX_ITERATIONS: usize = 100;

    fn create_random_participants(
        agent_seeds: &[u64; 6],
        num_players: u8,
    ) -> HashMap<u8, Box<dyn Agent>> {
        let mut participants: HashMap<u8, Box<dyn Agent>> = HashMap::new();
        for i in 0..num_players {
            participants.insert(i, Box::new(RandomAgent::new(Some(agent_seeds[i as usize]))));
        }
        participants
    }

    proptest! {
        #[test]
        fn test_game_termination(
            deck_seed in any::<u64>(),
            agent_seeds in any::<[u64; 6]>(),
            num_players in 3..=6u8
        ) {
            let participants = create_random_participants(&agent_seeds, num_players);
            let mut runner = GameRunner::new(participants, Some(deck_seed));

            let mut iterations = 0;
            while !runner.run_iteration() {
                iterations += 1;
                prop_assert!(iterations < MAX_ITERATIONS);
            }

            assert!(runner.state.get_is_game_over());
        }

        #[test]
        fn test_points_count(
            deck_seed in any::<u64>(),
            agent_seeds in any::<[u64; 6]>(),
            num_players in 3..=6u8
        ) {
            let participants = create_random_participants(&agent_seeds, num_players);
            let mut runner = GameRunner::new(participants, Some(deck_seed));

            let mut iterations = 0;
            while !runner.run_iteration() && iterations < MAX_ITERATIONS {
                iterations += 1;

                let uid = runner.state.get_current_player_uid();
                let view = runner.state.scrub_state(uid);
                assert!(view.available_points.len() as u8 <= view.active_player_count);
            }
        }
    }
}
