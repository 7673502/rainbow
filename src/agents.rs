use crate::game::Combo;
use crate::game::GameView;

pub trait Agent {
    fn choose_action(&self, view: GameView, valid_actions: Vec<Combo>) -> usize;
}
