use rand::prelude::*;
use std::cmp::Ordering;

const MAX_RANK: usize = 6;

#[derive(PartialEq)]
enum TrickType {
    Open,
    Set,
    Run,
}

enum Combo {
    Single { card: u8 },
    Set { card: u8, count: u8 },
    Run { start: u8, end: u8 },
}

impl PartialEq for Combo {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Combo::Single { card: x }, Combo::Single { card: y }) => x == y,
            (
                Combo::Set {
                    card: x,
                    count: x_count,
                },
                Combo::Set {
                    card: y,
                    count: y_count,
                },
            ) => x == y && x_count == y_count,
            (
                Combo::Run {
                    start: x_start,
                    end: x_end,
                },
                Combo::Run {
                    start: y_start,
                    end: y_end,
                },
            ) => x_start == y_start && x_end == y_end,
            _ => false,
        }
    }
}

impl PartialOrd for Combo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Combo::Single { .. }, Combo::Set { .. }) => Some(Ordering::Less),
            (Combo::Single { .. }, Combo::Run { .. }) => Some(Ordering::Less),
            (Combo::Set { .. }, Combo::Single { .. }) => Some(Ordering::Greater),
            (Combo::Run { .. }, Combo::Single { .. }) => Some(Ordering::Greater),
            (Combo::Single { card: x }, Combo::Single { card: y }) => x.partial_cmp(&y),
            (
                Combo::Set {
                    card: x,
                    count: x_count,
                },
                Combo::Set {
                    card: y,
                    count: y_count,
                },
            ) => Some(x_count.cmp(&y_count).then(x.cmp(&y))),
            (
                Combo::Run {
                    start: x_start,
                    end: x_end,
                },
                Combo::Run {
                    start: y_start,
                    end: y_end,
                },
            ) => {
                let x_len = x_end - x_start + 1;
                let y_len = y_end - y_start + 1;
                Some(x_len.cmp(&y_len).then(x_end.cmp(&y_end)))
            }
            _ => None,
        }
    }
}

struct Play {
    player_uid: u32,
    trick: Combo,
}

struct GameState {
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

struct Player {
    uid: u32,
    hand: [u8; MAX_RANK + 1],
    points: u8,
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

fn main() {
    let game = GameState::new(vec![1, 2, 3]);
    println!("{:?}", game.points);
}

#[cfg(test)]
mod get_legal_actions_tests {
    use super::*;

    #[test]
    fn test_all_cards() {
        let mut game = GameState::new(vec![1, 2, 3]);

        game.players[0].hand = [0, 2, 2, 0, 0, 0, 0];

        let legal_actions = game.get_legal_actions(1);

        assert!(legal_actions.contains(&Combo::Single { card: 1 }));
    }
}
