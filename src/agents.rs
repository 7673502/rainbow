use std::fmt::Debug;

use crate::game::Combo;
use crate::game::GameView;

pub trait Agent: Debug {
    fn choose_action(&self, view: GameView, valid_actions: &Vec<Combo>) -> usize;
}

pub mod random_agent;
