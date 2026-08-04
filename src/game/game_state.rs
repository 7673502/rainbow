use arrayvec::ArrayVec;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::cmp::Ordering;

use crate::constants::{
    DECK_SIZE, EMPTY_HANDS_TO_END_GAME, HAND_SIZE_3_4_PLAYERS, HAND_SIZE_5_PLAYERS,
    HAND_SIZE_6_PLAYERS, MAX_LEGAL_ACTIONS, MAX_PLAYERS, MAX_POINTS_VALUE, MAX_RANK, MIN_PLAYERS,
};
use crate::game::combo::{Combo, ComboKind};
use crate::game::maps::{PointMap, RankMap};
use crate::game::play::Play;
use crate::game::player::Player;
use crate::game::trick_type::TrickType;
use crate::game::view::{GameView, OpponentView};

#[derive(Debug)]
pub struct GameState {
    available_points: ArrayVec<u8, MAX_PLAYERS>,
    players: ArrayVec<Player, MAX_PLAYERS>,
    current_trick: ArrayVec<Play, MAX_PLAYERS>,
    current_trick_type: TrickType,
    current_player_index: u8,
    active_player_count: u8,
    is_game_over: bool,
}

impl GameState {
    pub fn new(player_uids: Vec<u8>, seed: Option<u64>) -> Self {
        let mut deck: Vec<u8> = (1..=MAX_RANK as u8).cycle().take(DECK_SIZE).collect();

        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(&mut rand::rng()),
        };
        deck.shuffle(&mut rng);

        let mut players: ArrayVec<Player, MAX_PLAYERS> =
            player_uids.iter().map(|p| Player::new(*p)).collect();

        let initial_hand_size = match players.len() {
            3 | 4 => HAND_SIZE_3_4_PLAYERS,
            5 => HAND_SIZE_5_PLAYERS,
            6 => HAND_SIZE_6_PLAYERS,
            _ => panic!(
                "The game supports {}-{} players (inclusive). Received player uids vector of length {}",
                MIN_PLAYERS,
                MAX_PLAYERS,
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
                players[i].add_cards(j, 1);
            }
        }

        let mut available_points = ArrayVec::<u8, MAX_PLAYERS>::new();
        for _ in 0..players.len() {
            available_points.push(
                deck.pop()
                    .expect("Deck empty when setting available points"),
            );
        }
        available_points.sort_unstable_by(|a, b| b.cmp(a));

        Self {
            available_points,
            players,
            current_trick: ArrayVec::<Play, MAX_PLAYERS>::new(),
            current_trick_type: TrickType::Open,
            current_player_index: 0,
            active_player_count: player_uids.len() as u8,
            is_game_over: false,
        }
    }

    fn player_by_uid(&self, player_uid: u8) -> &Player {
        self.players
            .iter()
            .find(|p| p.uid() == player_uid)
            .expect("Could not find player with given uid")
    }

    pub fn legal_actions(&self, player_uid: u8) -> ArrayVec<Combo, MAX_LEGAL_ACTIONS> {
        let mut combos = ArrayVec::<Combo, MAX_LEGAL_ACTIONS>::new();

        let player = self.player_by_uid(player_uid);

        for i in 1..=MAX_RANK as u8 {
            if player.rank_count(i) > 0 {
                combos.push(Combo::new_single(i));
                if self.current_trick_type != TrickType::Run {
                    for j in 2..=player.rank_count(i) {
                        combos.push(Combo::new_set(i, j));
                    }
                }
                if self.current_trick_type != TrickType::Set {
                    for j in i + 1..=MAX_RANK as u8 {
                        if player.rank_count(j) < 1 {
                            break;
                        }
                        combos.push(Combo::new_run(i, j));
                    }
                }
            }
        }

        combos
    }

    pub fn apply_action(&mut self, combo: Combo) {
        let current_player_uid = self.current_player_uid();

        // update the player hand
        let current_player = &mut self.players[self.current_player_index as usize];
        match combo.kind() {
            ComboKind::Single { rank } => {
                current_player.remove_cards(rank, 1);
            }
            ComboKind::Set { rank, count } => {
                current_player.remove_cards(rank, count);
                self.current_trick_type = TrickType::Set;
            }
            ComboKind::Run { start, end } => {
                (start..=end).for_each(|rank| {
                    current_player.remove_cards(rank, 1);
                });
                self.current_trick_type = TrickType::Run;
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
            if !self.players[self.current_player_index as usize].is_empty() {
                break;
            }
        }

        // end of round clean up
        if self.current_trick.len() as u8 == self.active_player_count {
            self.current_trick
                .sort_by(|a, b| b.combo.partial_cmp(&a.combo).unwrap_or(Ordering::Equal));
            // set next trick first player to winner
            let winner_uid = self
                .current_trick
                .first()
                .expect("trick vector empty at end of round")
                .player_uid;
            self.current_player_index = self
                .players
                .iter()
                .position(|p| p.uid() == winner_uid)
                .unwrap() as u8;
            for _ in 0..self.players.len() {
                if !self.players[self.current_player_index as usize].is_empty() {
                    break;
                }
                self.current_player_index =
                    (self.current_player_index + 1) % self.players.len() as u8;
            }

            // assign points
            for i in 0..self.available_points.len() {
                let play = &self.current_trick[i];
                let points = self.available_points[i];
                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.uid() == play.player_uid)
                    .expect("Could not find player with given uid");
                player.add_points(points);
            }

            // check if game is over
            let mut active_player_count = self.players.len() as u8;
            for p in &self.players {
                if p.is_empty() {
                    active_player_count -= 1;
                }
            }
            if active_player_count <= self.players.len() as u8 - EMPTY_HANDS_TO_END_GAME {
                self.is_game_over = true;
                return;
            }
            self.active_player_count = active_player_count;

            // update next round's points
            let mut total_counts = RankMap::new();
            for play in &self.current_trick {
                match play.combo.kind() {
                    ComboKind::Single { rank } => {
                        total_counts[rank] += 1;
                    }
                    ComboKind::Set { rank, count } => {
                        total_counts[rank] += count;
                    }
                    ComboKind::Run { start, end } => (start..=end).for_each(|rank| {
                        total_counts[rank] += 1;
                    }),
                }
            }
            let mut next_round_points_candidates = PointMap::new();
            for rank in 1..=MAX_RANK as u8 {
                let pairs = total_counts[rank] / 2;
                let singles = total_counts[rank] % 2;
                next_round_points_candidates[rank * 2] += pairs;
                next_round_points_candidates[rank] += singles;
            }
            self.available_points.clear();
            for value in (1..=MAX_POINTS_VALUE as u8).rev() {
                while next_round_points_candidates[value] > 0
                    && self.available_points.len() < self.active_player_count as usize
                {
                    self.available_points.push(value);
                    next_round_points_candidates[value] -= 1;
                }
            }

            // miscellaneous clean up tasks
            self.current_trick_type = TrickType::Open;
            self.current_trick.clear();
        }
    }

    pub fn scrub_state(&self, player_uid: u8) -> GameView {
        let player = self.player_by_uid(player_uid);

        let mut opponents = ArrayVec::<OpponentView, { MAX_PLAYERS - 1 }>::new();

        for p in &self.players {
            if p.uid() != player_uid {
                opponents.push(OpponentView {
                    uid: p.uid(),
                    hand_size: p.hand_size(),
                    points: p.points(),
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
            my_hand: player.hand(),
            my_points: player.points(),

            opponents,
        }
    }

    pub fn current_player_uid(&self) -> u8 {
        self.players[self.current_player_index as usize].uid()
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_new() {
        for num_players in 3..=6 {
            let game = GameState::new((1..=num_players).collect(), Some(42));

            assert_eq!(game.available_points.len(), num_players as usize);
            assert_eq!(game.players.len(), num_players as usize);
            assert!(game.current_trick.is_empty());
            assert_eq!(game.current_trick_type, TrickType::Open);
            assert_eq!(game.current_player_index, 0);
            assert_eq!(game.active_player_count, num_players);
            assert!(!game.is_game_over);
        }
    }

    #[test]
    fn test_legal_actions_singles() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([2, 2, 0, 0, 0, 0]);
            game.current_trick_type = TrickType::Run;

            let legal_actions = game.legal_actions(1);

            assert_eq!(legal_actions.len(), 3);
            assert!(legal_actions.contains(&Combo::new_single(1)));
            assert!(legal_actions.contains(&Combo::new_single(2)));
            assert!(!legal_actions.contains(&Combo::new_single(3)));
            assert!(!legal_actions.contains(&Combo::new_single(4)));
            assert!(!legal_actions.contains(&Combo::new_single(5)));
            assert!(!legal_actions.contains(&Combo::new_single(6)));
        }
    }

    #[test]
    fn test_legal_actions_sets() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([0, 3, 0, 3, 3, 0]);
            game.current_trick_type = TrickType::Set;

            let legal_actions = game.legal_actions(1);

            assert_eq!(legal_actions.len(), 9);

            assert!(!legal_actions.contains(&Combo::new_single(1)));
            assert!(legal_actions.contains(&Combo::new_single(2)));
            assert!(!legal_actions.contains(&Combo::new_single(3)));
            assert!(legal_actions.contains(&Combo::new_single(4)));
            assert!(legal_actions.contains(&Combo::new_single(5)));
            assert!(!legal_actions.contains(&Combo::new_single(6)));

            assert!(legal_actions.contains(&Combo::new_set(2, 2)));
            assert!(legal_actions.contains(&Combo::new_set(2, 3)));
            assert!(legal_actions.contains(&Combo::new_set(4, 2)));
            assert!(legal_actions.contains(&Combo::new_set(4, 3)));
            assert!(legal_actions.contains(&Combo::new_set(5, 2)));
            assert!(legal_actions.contains(&Combo::new_set(5, 3)));

            assert!(!legal_actions.contains(&Combo::new_run(4, 5)));
        }
    }

    #[test]
    fn test_legal_actions_runs() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([2, 3, 1, 3, 0, 3]);
            game.current_trick_type = TrickType::Run;

            let legal_actions = game.legal_actions(1);

            assert_eq!(legal_actions.len(), 11);

            assert!(legal_actions.contains(&Combo::new_single(1)));
            assert!(legal_actions.contains(&Combo::new_single(2)));
            assert!(legal_actions.contains(&Combo::new_single(3)));
            assert!(legal_actions.contains(&Combo::new_single(4)));
            assert!(!legal_actions.contains(&Combo::new_single(5)));
            assert!(legal_actions.contains(&Combo::new_single(6)));

            assert!(legal_actions.contains(&Combo::new_run(1, 4)));
            assert!(!legal_actions.contains(&Combo::new_run(1, 5)));
        }
    }

    #[test]
    fn test_legal_actions_open() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([0, 0, 2, 2, 0, 0]);
            assert_eq!(game.current_trick_type, TrickType::Open);

            let legal_actions = game.legal_actions(1);

            assert_eq!(legal_actions.len(), 5);

            assert!(legal_actions.contains(&Combo::new_single(3)));
            assert!(legal_actions.contains(&Combo::new_single(4)));
        }
    }

    #[test]
    fn test_apply_action_to_single() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([0, 0, 0, 0, 2, 0]);
            assert_eq!(game.current_trick_type, TrickType::Open);

            game.apply_action(Combo::new_single(5));
            assert_eq!(game.current_trick_type, TrickType::Open);
            assert_eq!(game.players[0].rank_count(5), 1);
        }
    }

    #[test]
    fn test_apply_action_to_set() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([0, 0, 0, 0, 0, 3]);
            assert_eq!(game.current_trick_type, TrickType::Open);

            game.apply_action(Combo::new_set(6, 2));
            assert_eq!(game.current_trick_type, TrickType::Set);
            assert_eq!(game.players[0].rank_count(6), 1);
        }
    }

    #[test]
    fn test_apply_action_to_run() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([1, 2, 1, 1, 0, 0]);
            assert_eq!(game.current_trick_type, TrickType::Open);

            game.apply_action(Combo::new_run(1, 4));
            assert_eq!(game.current_trick_type, TrickType::Run);
            assert_eq!(game.players[0].rank_count(1), 0);
            assert_eq!(game.players[0].rank_count(2), 1);
            assert_eq!(game.players[0].rank_count(3), 0);
            assert_eq!(game.players[0].rank_count(4), 0);
            assert_eq!(game.players[0].rank_count(5), 0);
            assert_eq!(game.players[0].rank_count(6), 0);
        }
    }

    #[test]
    fn test_empty_hand_inactivity() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            game.players[0].set_hand([1, 1, 1, 1, 1, 1]);
            game.apply_action(Combo::new_run(1, 6));
            for i in 1..num_players as usize {
                game.players[i].set_hand([2, 2, 2, 2, 2, 2]);
                game.apply_action(Combo::new_single(2));
            }

            assert_eq!(game.current_player_index, 1);
            assert_eq!(game.active_player_count, num_players - 1);
        }
    }

    #[test]
    fn test_scrub_state() {
        for num_players in 3..=6 {
            let mut game = GameState::new((1..=num_players).collect(), Some(42));

            // give players specific points to differentiate them
            for (i, p) in game.players.iter_mut().enumerate() {
                p.add_points((i as u8 + 1) * 10);
            }

            let my_uid = 1;
            let view = game.scrub_state(my_uid);

            // verify personal data is fully intact
            assert_eq!(view.my_uid, my_uid);
            assert_eq!(view.my_points, 10); // Player 1 (index 0) gets 10 points
            assert_eq!(view.my_hand, game.players[0].hand());

            // verify global state is correctly copied
            assert_eq!(view.available_points, game.available_points);
            assert_eq!(view.current_trick_type, game.current_trick_type);
            assert_eq!(view.current_player_index, game.current_player_index);
            assert_eq!(view.active_player_count, game.active_player_count);

            // verify opponents are scrubbed (no hand contents, just sizes)
            assert_eq!(view.opponents.len(), num_players as usize - 1);
            for opponent_view in view.opponents {
                // ensure the player calling scrub_state is NOT in the opponents list
                assert_ne!(opponent_view.uid, my_uid);

                let original_player = game
                    .players
                    .iter()
                    .find(|p| p.uid() == opponent_view.uid)
                    .unwrap();

                assert_eq!(opponent_view.hand_size, original_player.hand_size());
                assert_eq!(opponent_view.points, original_player.points());
            }
        }
    }

    #[test]
    fn test_trick_resolution() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        // give players specific cards
        game.players[0].set_hand([1, 1, 0, 0, 0, 0]);
        game.players[1].set_hand([1, 0, 0, 0, 1, 0]);
        game.players[2].set_hand([1, 0, 0, 1, 0, 0]);

        // play cards in turn order
        game.apply_action(Combo::new_single(2));
        game.apply_action(Combo::new_single(5));
        game.apply_action(Combo::new_single(4));

        // p2 played the 5, so p2 wins and goes next
        assert_eq!(game.current_player_index, 1);

        // trick state should reset
        assert!(game.current_trick.is_empty());
        assert_eq!(game.current_trick_type, TrickType::Open);
    }

    #[test]
    fn test_trick_resolution_tie() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        // give players cards with a tie for highest
        game.players[0].set_hand([1, 0, 0, 0, 0, 1]);
        game.players[1].set_hand([1, 0, 0, 1, 0, 0]);
        game.players[2].set_hand([1, 0, 0, 0, 0, 1]);

        // play the cards
        game.apply_action(Combo::new_single(6));
        game.apply_action(Combo::new_single(4));
        game.apply_action(Combo::new_single(6));

        // p1 played the 6 first, so p1 should win the tie
        assert_eq!(game.current_player_index, 0);
    }

    #[test]
    fn test_next_round_points() {
        let mut game = GameState::new(vec![1, 2, 3], Some(42));

        // empty hands to control exactly what is played
        for p in &mut game.players {
            p.set_hand([0; MAX_RANK]);
        }

        // setup players so they can play two 3s and two 4s total
        game.players[0].set_hand([1, 0, 3, 0, 0, 0]);
        game.players[1].set_hand([1, 0, 0, 2, 0, 0]);
        game.players[2].set_hand([1, 0, 0, 2, 0, 0]);

        // play the cards
        game.apply_action(Combo::new_set(3, 2));
        game.apply_action(Combo::new_single(4));
        game.apply_action(Combo::new_single(4));

        // the trick had two 3s and two 4s.
        println!("{:?}", game.available_points);
        assert!(game.available_points.contains(&8));
        assert!(game.available_points.contains(&6));
    }

    #[test]
    fn test_game_over_trigger() {
        let mut game = GameState::new(vec![1, 2, 3, 4], Some(42));

        for i in 0..4 {
            game.players[i].set_hand([0; MAX_RANK]);
        }

        game.players[0].set_hand([1, 0, 0, 0, 0, 0]);
        game.players[1].set_hand([2, 0, 0, 0, 0, 0]);
        game.players[2].set_hand([1, 0, 0, 0, 0, 0]);
        game.players[3].set_hand([3, 0, 0, 0, 0, 0]);

        game.apply_action(Combo::new_single(1));
        game.apply_action(Combo::new_single(1));
        game.apply_action(Combo::new_single(1));
        game.apply_action(Combo::new_single(1));

        assert!(game.is_game_over);
    }
}
