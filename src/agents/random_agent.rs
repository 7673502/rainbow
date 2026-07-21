use crate::agents::Agent;
use crate::{Combo, GameView};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::cell::RefCell;

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
    fn choose_action(&self, _view: GameView, valid_actions: &Vec<Combo>) -> usize {
        let mut rng = self.rng.borrow_mut();
        rng.random_range(0..valid_actions.len())
    }
}
