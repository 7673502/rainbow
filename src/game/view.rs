use crate::constants::MAX_RANK;
use crate::game::play::Play;
use crate::game::trick_type::TrickType;

pub struct GameView {
    pub available_points: Vec<u8>,
    pub current_trick: Vec<Play>,
    pub current_trick_type: TrickType,
    pub current_player_index: u8,
    pub active_player_count: u8,

    pub my_uid: u8,
    pub my_hand: [u8; MAX_RANK + 1],
    pub my_points: u8,

    pub opponents: Vec<OpponentView>,
}

pub struct OpponentView {
    pub uid: u8,
    pub hand_size: u8,
    pub points: u8,
}
