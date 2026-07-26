use crate::agents::Agent;
use crate::{Combo, GameView};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::cell::RefCell;

#[derive(Debug)]
pub struct RandomAgent {
    rng: RefCell<StdRng>,
}

impl RandomAgent {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(&mut rand::rng()),
        };

        Self {
            rng: RefCell::new(rng),
        }
    }
}

impl Agent for RandomAgent {
    fn choose_action(&self, _view: GameView, valid_actions: &[Combo]) -> usize {
        let mut rng = self.rng.borrow_mut();
        rng.random_range(0..valid_actions.len())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_choose_action_valid_index(
            seed in any::<u64>(),
            valid_actions in vec(any::<Combo>(), 1..10),
        ) {
            let agent = RandomAgent::new(Some(seed));
            let dummy_view = crate::game::game_state::GameState::new(vec![1, 2, 3], None).scrub_state(1);
            let index = agent.choose_action(dummy_view, &valid_actions);

            prop_assert!(index < valid_actions.len());
        }
    }
}
