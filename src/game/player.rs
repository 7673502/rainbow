use crate::constants::{MAX_COUNT_PER_RANK, MAX_RANK, check_count, check_rank};
use crate::game::rank_map::RankMap;

#[derive(Debug)]
pub struct Player {
    uid: u8,
    hand: RankMap,
    points: u8,
}

impl Player {
    pub fn new(uid: u8) -> Self {
        Self {
            uid,
            hand: RankMap::new(),
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

    pub fn hand(&self) -> RankMap {
        self.hand
    }

    pub fn rank_count(&self, rank: u8) -> u8 {
        check_rank(rank);
        self.hand[rank]
    }

    pub fn add_cards(&mut self, rank: u8, count: u8) {
        check_rank(rank);
        check_count(count);
        assert!(
            count + self.hand[rank] <= MAX_COUNT_PER_RANK as u8,
            "Adding {} cards when player hand contains {} cards of rank {} exceeds max count per rank of {}",
            count,
            self.hand[rank],
            rank,
            MAX_COUNT_PER_RANK
        );
        self.hand[rank] += count;
    }

    pub fn remove_cards(&mut self, rank: u8, count: u8) {
        check_rank(rank);
        check_count(count);
        assert!(
            count <= self.hand[rank],
            "Count must be less than or equal to amount in hand"
        );
        self.hand[rank] -= count;
    }

    pub fn hand_size(&self) -> u8 {
        let mut total = 0;
        for rank in 1..=MAX_RANK as u8 {
            total += self.hand[rank];
        }
        total
    }

    pub fn is_empty(&self) -> bool {
        self.hand_size() == 0
    }

    #[cfg(test)]
    pub fn set_hand(&mut self, hand: [u8; MAX_RANK]) {
        self.hand = RankMap::new();

        for rank in 1..=MAX_RANK {
            self.hand[rank as u8] = hand[rank - 1];
        }
    }
}
