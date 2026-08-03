use crate::game::play::Play;
use crate::game::rank_map::RankMap;
use crate::game::trick_type::TrickType;

#[derive(Debug)]
pub struct GameView {
    pub available_points: Vec<u8>,
    pub current_trick: Vec<Play>,
    pub current_trick_type: TrickType,
    pub current_player_index: u8,
    pub active_player_count: u8,

    pub my_uid: u8,
    pub my_hand: RankMap,
    pub my_points: u8,

    pub opponents: Vec<OpponentView>,
}

#[derive(Debug)]
pub struct OpponentView {
    pub uid: u8,
    pub hand_size: u8,
    pub points: u8,
}
