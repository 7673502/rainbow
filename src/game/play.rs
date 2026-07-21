use crate::game::combo::Combo;

#[derive(Debug, Copy, Clone)]
pub struct Play {
    pub player_uid: u8,
    pub combo: Combo,
}
