use rainbow::agents::Agent;
use rainbow::agents::random_agent::RandomAgent;
use rainbow::game::game_runner::GameRunner;

fn main() {
    let mut participants: Vec<Box<dyn Agent>> = Vec::new();
    for _ in 0..3 {
        participants.push(Box::new(RandomAgent::new(None)));
    }
    let mut runner = GameRunner::new(participants, Some(42));

    let final_state = runner.run_game();

    println!("{}", final_state.is_game_over());
}
