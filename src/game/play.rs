use crate::game::combo::Combo;

#[derive(Copy, Clone)]
pub struct Play {
    pub player_uid: u8,
    pub combo: Combo,
}
