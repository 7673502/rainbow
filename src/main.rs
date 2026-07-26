use rainbow::agents::Agent;
use rainbow::agents::random_agent::RandomAgent;
use rainbow::game::game_runner::GameRunner;
use std::collections::HashMap;

fn main() {
    let mut participants: HashMap<u8, Box<dyn Agent>> = HashMap::new();
    for uid in 1..=3u8 {
        participants.insert(uid, Box::new(RandomAgent::new(None)));
    }
    let mut runner = GameRunner::new(participants, Some(42));

    let final_state = runner.run_game();

    println!("{}", final_state.is_game_over());
}
