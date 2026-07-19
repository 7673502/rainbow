use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::combo::Combo;
use crate::constants::MAX_RANK;
use crate::play::Play;
use crate::player::Player;
use crate::trick_type::TrickType;
use crate::view::{GameView, OpponentView};

pub struct GameState {
    available_points: Vec<u8>,
    players: Vec<Player>,
    current_trick: Vec<Play>,
    current_trick_type: TrickType,
    current_player_index: u8,
    active_player_count: u8,
}

impl GameState {
    pub fn new(player_uids: Vec<u8>, seed: u64) -> Self {
        let cards: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        let mut deck: Vec<u8> = cards.into_iter().cycle().take(60).collect();
        deck.shuffle(&mut StdRng::seed_from_u64(seed));

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_legal_actions_singles() {
        let mut game = GameState::new(vec![1, 2, 3], 42);

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
        let mut game = GameState::new(vec![1, 2, 3], 42);

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
        let mut game = GameState::new(vec![1, 2, 3], 42);

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
