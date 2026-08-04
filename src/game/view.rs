use arrayvec::ArrayVec;

use crate::constants::MAX_PLAYERS;
use crate::game::maps::RankMap;
use crate::game::play::Play;
use crate::game::trick_type::TrickType;

#[derive(Debug)]
pub struct GameView {
    pub available_points: ArrayVec<u8, MAX_PLAYERS>,
    pub current_trick: ArrayVec<Play, MAX_PLAYERS>,
    pub current_trick_type: TrickType,
    pub current_player_index: u8,
    pub active_player_count: u8,

    pub my_uid: u8,
    pub my_hand: RankMap,
    pub my_points: u8,

    pub opponents: ArrayVec<OpponentView, { MAX_PLAYERS - 1 }>,
}

#[derive(Debug)]
pub struct OpponentView {
    pub uid: u8,
    pub hand_size: u8,
    pub points: u8,
}
