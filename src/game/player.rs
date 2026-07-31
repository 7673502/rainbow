use crate::constants::{MAX_COUNT_PER_RANK, MAX_RANK};

#[derive(Debug)]
pub struct Player {
    pub uid: u8,
    hand: [u8; MAX_RANK + 1],
    pub points: u8,
}

impl Player {
    pub fn new(uid: u8) -> Self {
        Self {
            uid,
            hand: [0; MAX_RANK + 1],
            points: 0,
        }
    }

    pub fn hand(&self) -> [u8; MAX_RANK + 1] {
        self.hand
    }

    pub fn rank_count(&self, rank: u8) -> u8 {
        Self::check_rank(rank);
        self.hand[rank as usize]
    }

    pub fn add_cards(&mut self, rank: u8, count: u8) {
        Self::check_rank(rank);
        Self::check_count(count);
        assert!(
            count + self.hand[rank as usize] <= MAX_COUNT_PER_RANK as u8,
            "Adding {} cards when player hand contains {} cards of rank {} exceeds max count per rank of {}",
            count,
            self.hand[rank as usize],
            rank,
            MAX_COUNT_PER_RANK
        );
        self.hand[rank as usize] += count;
    }

    pub fn remove_cards(&mut self, rank: u8, count: u8) {
        Self::check_rank(rank);
        Self::check_count(count);
        assert!(
            count <= self.hand[rank as usize],
            "Count must be less than or equal to amount in hand"
        );
        self.hand[rank as usize] -= count;
    }

    pub fn hand_size(&self) -> u8 {
        self.hand.iter().sum::<u8>()
    }

    pub fn is_empty(&self) -> bool {
        self.hand_size() == 0
    }

    fn check_rank(rank: u8) {
        assert!(
            1 <= rank && rank <= MAX_RANK as u8,
            "Rank must be between 1 and {} inclusive",
            MAX_RANK
        );
    }

    fn check_count(count: u8) {
        assert!(
            1 <= count && count <= MAX_COUNT_PER_RANK as u8,
            "Count must be between 1 and {} inclusive",
            MAX_COUNT_PER_RANK
        );
    }

    #[cfg(test)]
    pub fn set_hand(&mut self, hand: [u8; MAX_RANK + 1]) {
        self.hand = hand;
    }
}
