use crate::game::combo::Combo;

#[derive(Copy, Clone)]
pub struct Play {
    player_uid: u8,
    combo: Combo,
}
