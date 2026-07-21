use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::cmp::Ordering;

use crate::constants::MAX_RANK;
use crate::game::combo::Combo;
use crate::game::play::Play;
use crate::game::player::Player;
use crate::game::trick_type::TrickType;
use crate::game::view::{GameView, OpponentView};

#[derive(Debug)]
pub struct GameState {
    available_points: Vec<u8>,
    players: Vec<Player>,
    current_trick: Vec<Play>,
    current_trick_type: TrickType,
    current_player_index: u8,
    active_player_count: u8,
    is_game_over: bool,
}

impl GameState {
    pub fn new(player_uids: Vec<u8>, seed: Option<u64>) -> Self {
        let cards: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        let mut deck: Vec<u8> = cards.into_iter().cycle().take(60).collect();

        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(&mut rand::rng()),
        };
        deck.shuffle(&mut rng);

        let mut players: Vec<Player> = player_uids.iter().map(|p| Player::new(*p)).collect();

        let initial_hand_size = match players.len() {
            3..=4 => 14,
            5 => 11,
            6 => 9,
            _ => panic!(
                "The game supports 3-6 players (inclusive). Received player uids vector of length {}",
                players.len()
            ),
        };

        for i in 0..players.len() {
            if player_uids[i + 1..].contains(&player_uids[i]) {
                panic!(
                    "All player uids must be unique. Received uid {} more than once",
                    player_uids[i]
                )
            }

            let hand_vec = deck.split_off(deck.len() - initial_hand_size);
            for j in hand_vec {
                players[i].hand[j as usize] += 1;
            }
        }

        let mut available_points: Vec<u8> = deck.split_off(deck.len() - players.len());
        available_points.sort_unstable();

        Self {
            available_points,
            players,
            current_trick: Vec::new(),
            current_trick_type: TrickType::Open,
            current_player_index: 0,
            active_player_count: player_uids.len() as u8,
            is_game_over: false,
        }
    }

    fn get_player(&self, player_uid: u8) -> &Player {
        &self
            .players
            .iter()
            .find(|p| p.uid == player_uid)
            .expect("Could not find player with given uid")
    }

    pub fn get_legal_actions(&self, player_uid: u8) -> Vec<Combo> {
        let mut combos = Vec::new();

        let player = self.get_player(player_uid);

        for i in 1..=MAX_RANK {
            if player.hand[i] > 0 {
                combos.push(Combo::Single { card: i as u8 });
                if self.current_trick_type != TrickType::Run {
                    for j in 2..=player.hand[i] {
                        combos.push(Combo::Set {
                            card: i as u8,
                            count: j,
                        });
                    }
                }
                if self.current_trick_type != TrickType::Set {
                    for j in i + 1..=MAX_RANK {
                        if player.hand[j] < 1 {
                            break;
                        }
                        combos.push(Combo::Run {
                            start: i as u8,
                            end: j as u8,
                        });
                    }
                }
            }
        }

        combos
    }

    pub fn apply_action(&mut self, combo: Combo) {
        let current_player_uid = self.get_current_player_uid();

        // update the player hand
        let current_player = &mut self.players[self.current_player_index as usize];
        match &combo {
            &Combo::Single { card } => {
                current_player.hand[card as usize] -= 1;
            }
            &Combo::Set { card, count } => {
                current_player.hand[card as usize] -= count;
            }
            &Combo::Run { start, end } => {
                (start..=end).for_each(|card| {
                    current_player.hand[card as usize] -= 1;
                });
            }
        };

        // update the current trick
        let play = Play {
            player_uid: current_player_uid,
            combo,
        };
        self.current_trick.push(play);

        // update current player
        for _ in 0..self.players.len() {
            self.current_player_index = (self.current_player_index + 1) % self.players.len() as u8;
            if self.players[self.current_player_index as usize]
                .hand
                .iter()
                .sum::<u8>()
                > 0
            {
                break;
            }
        }

        // end of round clean up
        if self.current_trick.len() as u8 == self.active_player_count {
            self.current_trick
                .sort_by(|a, b| a.combo.partial_cmp(&b.combo).unwrap_or(Ordering::Equal));
            // set next trick first player to winner
            let winner_uid = self
                .current_trick
                .last()
                .expect("trick vector empty at end of round")
                .player_uid;
            self.current_player_index = self
                .players
                .iter()
                .position(|p| p.uid == winner_uid)
                .unwrap() as u8;
            for _ in 0..self.players.len() {
                if self.players[self.current_player_index as usize]
                    .hand
                    .iter()
                    .sum::<u8>()
                    > 0
                {
                    break;
                }
                self.current_player_index =
                    (self.current_player_index + 1) % self.players.len() as u8;
            }

            // assign points
            for play in &self.current_trick {
                if let Some(points) = self.available_points.pop() {
                    if let Some(player) = self.players.iter_mut().find(|p| p.uid == play.player_uid)
                    {
                        player.points += points;
                    }
                } else {
                    break;
                }
            }

            // check if game is over
            let mut active_player_count = self.players.len() as u8;
            for p in &self.players {
                if p.hand.iter().sum::<u8>() == 0 {
                    active_player_count -= 1;
                }
            }
            if active_player_count <= self.players.len() as u8 - 2 {
                self.is_game_over = true;
                return;
            }
            self.active_player_count = active_player_count;

            // update next round's points
            let mut total_counts = [0; MAX_RANK + 1];
            for play in &self.current_trick {
                match play.combo {
                    Combo::Single { card } => {
                        total_counts[card as usize] += 1;
                    }
                    Combo::Set { card, count } => {
                        total_counts[card as usize] += count;
                    }
                    Combo::Run { start, end } => (start..=end).for_each(|rank| {
                        total_counts[rank as usize] += 1;
                    }),
                }
            }
            let mut next_round_points_candidates = [0u8; 2 * MAX_RANK + 1];
            for rank in 1..=MAX_RANK {
                let pairs = total_counts[rank] / 2;
                let singles = total_counts[rank] % 2;
                next_round_points_candidates[rank * 2] += pairs;
                next_round_points_candidates[rank] += singles;
            }
            self.available_points.clear();
            for value in (1..=12).rev() {
                while next_round_points_candidates[value] > 0
                    && self.available_points.len() < self.active_player_count as usize
                {
                    self.available_points.push(value as u8);
                }
            }

            // miscellaneous clean up tasks
            self.current_trick_type = TrickType::Open;
            self.current_trick.clear();
        }
    }

    pub fn scrub_state(&self, player_uid: u8) -> GameView {
        let player = self.get_player(player_uid);

        let mut opponents: Vec<OpponentView> = Vec::new();

        for p in &self.players {
            if p.uid != player_uid {
                opponents.push(OpponentView {
                    uid: p.uid,
                    hand_size: p.hand.len() as u8,
                    points: p.points,
                });
            }
        }

        GameView {
            available_points: self.available_points.clone(),
            current_trick: self.current_trick.clone(),
            current_trick_type: self.current_trick_type,
            current_player_index: self.current_player_index,
            active_player_count: self.active_player_count,

            my_uid: player_uid,
            my_hand: player.hand.clone(),
            my_points: player.points,

            opponents,
        }
    }

    pub fn get_current_player_uid(&self) -> u8 {
        self.players[self.current_player_index as usize].uid
    }

    pub fn get_is_game_over(&self) -> bool {
        return self.is_game_over;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_get_legal_actions_singles() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        game.players[0].hand = [0, 2, 2, 0, 0, 0, 0];
        game.current_trick_type = TrickType::Run;

        let legal_actions = game.get_legal_actions(1);

        assert_eq!(legal_actions.len(), 3);
        assert!(legal_actions.contains(&Combo::Single { card: 1 }));
        assert!(legal_actions.contains(&Combo::Single { card: 2 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 3 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 4 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 5 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 6 }));
    }

    #[test]
    fn test_get_legal_actions_sets() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        game.players[0].hand = [0, 0, 3, 0, 3, 3, 0];
        game.current_trick_type = TrickType::Set;

        let legal_actions = game.get_legal_actions(1);

        assert_eq!(legal_actions.len(), 9);

        assert!(!legal_actions.contains(&Combo::Single { card: 1 }));
        assert!(legal_actions.contains(&Combo::Single { card: 2 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 3 }));
        assert!(legal_actions.contains(&Combo::Single { card: 4 }));
        assert!(legal_actions.contains(&Combo::Single { card: 5 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 6 }));

        assert!(legal_actions.contains(&Combo::Set { card: 2, count: 2 }));
        assert!(legal_actions.contains(&Combo::Set { card: 2, count: 3 }));
        assert!(legal_actions.contains(&Combo::Set { card: 4, count: 2 }));
        assert!(legal_actions.contains(&Combo::Set { card: 4, count: 3 }));
        assert!(legal_actions.contains(&Combo::Set { card: 5, count: 2 }));
        assert!(legal_actions.contains(&Combo::Set { card: 5, count: 3 }));

        assert!(!legal_actions.contains(&Combo::Run { start: 4, end: 5 }));
        assert!(!legal_actions.contains(&Combo::Set { card: 2, count: 1 }));
    }

    #[test]
    fn test_get_legal_actions_runs() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        game.players[0].hand = [0, 2, 3, 1, 3, 0, 3];
        game.current_trick_type = TrickType::Run;

        let legal_actions = game.get_legal_actions(1);

        assert_eq!(legal_actions.len(), 11);

        assert!(legal_actions.contains(&Combo::Single { card: 1 }));
        assert!(legal_actions.contains(&Combo::Single { card: 2 }));
        assert!(legal_actions.contains(&Combo::Single { card: 3 }));
        assert!(legal_actions.contains(&Combo::Single { card: 4 }));
        assert!(!legal_actions.contains(&Combo::Single { card: 5 }));
        assert!(legal_actions.contains(&Combo::Single { card: 6 }));

        assert!(legal_actions.contains(&Combo::Run { start: 1, end: 4 }));
        assert!(!legal_actions.contains(&Combo::Run { start: 1, end: 5 }));
        assert!(!legal_actions.contains(&Combo::Set { card: 4, count: 3 }))
    }
}
