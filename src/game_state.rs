use rand::prelude::*;

use crate::combo::Combo;
use crate::constants::MAX_RANK;
use crate::play::Play;
use crate::player::Player;
use crate::trick_type::TrickType;

pub struct GameState {
    points: Vec<u8>,
    players: Vec<Player>,
    current_trick: Vec<Play>,
    current_trick_type: TrickType,
    current_player_index: usize,
    active_players: u8,
}

impl GameState {
    pub fn new(player_uids: Vec<u32>) -> Self {
        let cards: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        let mut deck: Vec<u8> = cards.into_iter().cycle().take(60).collect();
        deck.shuffle(&mut rand::rng());

        let mut players: Vec<Player> = player_uids.iter().map(|p| Player::new(*p)).collect();

        let initial_hand_size = match players.len() {
            3..=4 => 14,
            5 => 11,
            6 => 9,
            _ => panic!(
                "Game supports 3-6 players inclusive. Received player uids vector of length {}",
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

        let mut points: Vec<u8> = deck.split_off(deck.len() - players.len());
        points.sort_unstable();

        Self {
            points,
            players,
            current_trick: Vec::new(),
            current_trick_type: TrickType::Open,
            current_player_index: 0,
            active_players: player_uids.len() as u8,
        }
    }

    pub fn get_legal_actions(&self, player_uid: u32) -> Vec<Combo> {
        let mut combos = Vec::new();

        let player = self
            .players
            .iter()
            .find(|p| p.uid == player_uid)
            .expect("Could not find player with given uid");

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_cards() {
        let mut game = GameState::new(vec![1, 2, 3]);

        game.players[0].hand = [0, 2, 2, 0, 0, 0, 0];

        let legal_actions = game.get_legal_actions(1);

        assert!(legal_actions.contains(&Combo::Single { card: 1 }));
    }
}
