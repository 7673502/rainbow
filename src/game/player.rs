use crate::constants::{MAX_COUNT_PER_RANK, MAX_RANK, check_count, check_rank};

#[derive(Debug)]
pub struct Player {
    uid: u8,
    hand: [u8; MAX_RANK + 1],
    points: u8,
}

impl Player {
    pub fn new(uid: u8) -> Self {
        Self {
            uid,
            hand: [0; MAX_RANK + 1],
            points: 0,
        }
    }

    pub fn uid(&self) -> u8 {
        self.uid
    }

    pub fn points(&self) -> u8 {
        self.points
    }

    pub fn add_points(&mut self, points: u8) {
        self.points += points;
    }

    pub fn hand(&self) -> [u8; MAX_RANK + 1] {
        self.hand
    }

    pub fn rank_count(&self, rank: u8) -> u8 {
        check_rank(rank);
        self.hand[rank as usize]
    }

    pub fn add_cards(&mut self, rank: u8, count: u8) {
        check_rank(rank);
        check_count(count);
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
        check_rank(rank);
        check_count(count);
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

    #[cfg(test)]
    pub fn set_hand(&mut self, hand: [u8; MAX_RANK + 1]) {
        self.hand = hand;
    }
}
