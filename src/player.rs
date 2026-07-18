use crate::constants::MAX_RANK;

pub struct Player {
    pub uid: u32,
    pub hand: [u8; MAX_RANK + 1],
    pub points: u8,
}

impl Player {
    pub fn new(uid: u32) -> Self {
        Self {
            uid,
            hand: [0; MAX_RANK + 1],
            points: 0,
        }
    }
}
