mod dice_game;
use dice_game::*;
use rand::{thread_rng, Rng};
use std::thread;

fn strategy_random(board: &Board, state: &GameState, _: &usize) -> usize {
    let mut valid_answers = Vec::new();

    match state {
        GameState::RedsTurn => {
            for i in 0..board.columns {
                if board.has_space(&Team::Red, i) {
                    valid_answers.push(i);
                }
            }
        }
        GameState::BluesTurn => {
            for i in 0..board.columns {
                if board.has_space(&Team::Blue, i) {
                    valid_answers.push(i);
                }
            }
        }
        _ => panic!("Cannot take a turn in this game state."),
    }

    let index = thread_rng().gen_range(0..valid_answers.len());
    valid_answers[index]
}

fn strategy_min_max(board: &Board, state: &GameState, roll: &usize) -> usize {
    //println!("started min max strat");
    let mut valid_answers = Vec::new();
    let team = match state {
        GameState::RedsTurn => Team::Red,
        GameState::BluesTurn => Team::Blue,
        _ => panic!("Cannot take a turn in this game state."),
    };

    for i in 0..board.columns {
        if board.has_space(&team, i) {
            valid_answers.push(i);
        }
    }
    let index = thread_rng().gen_range(0..valid_answers.len());

    let mut best_answer = valid_answers[index];
    let mut best_improvement: isize = *roll as isize;
    for answer in valid_answers {
        let mut board_copy = board.clone();
        board_copy.insert(&team, answer, *roll);

        let own_score;
        let opposite_score;
        let new_own_score;
        let new_opposite_score;

        match team {
            Team::Red => {
                own_score = board.calculate_score(&Team::Red) as isize;
                opposite_score = board.calculate_score(&Team::Blue) as isize;
                new_own_score = board_copy.calculate_score(&Team::Red) as isize;
                new_opposite_score = board_copy.calculate_score(&Team::Blue) as isize;
            }
            Team::Blue => {
                own_score = board.calculate_score(&Team::Blue) as isize;
                opposite_score = board.calculate_score(&Team::Red) as isize;
                new_own_score = board_copy.calculate_score(&Team::Blue) as isize;
                new_opposite_score = board_copy.calculate_score(&Team::Red) as isize;
            }
        }

        let current_diff: isize = own_score - opposite_score;
        let future_diff: isize = new_own_score - new_opposite_score;
        let improvement: isize = future_diff - current_diff;
        if improvement > best_improvement {
            best_improvement = improvement;
            best_answer = answer;
        }
    }

    best_answer
}

struct TotalWins {
    red_wins: usize,
    blue_wins: usize,
    ties: usize,
}

impl TotalWins {
    fn new() -> Self {
        TotalWins {
            red_wins: 0,
            blue_wins: 0,
            ties: 0,
        }
    }

    fn add(&self, other: TotalWins) -> Self {
        TotalWins {
            red_wins: self.red_wins + other.red_wins,
            blue_wins: self.blue_wins + other.blue_wins,
            ties: self.ties + other.ties,
        }
    }
}

fn run_games(times: usize) -> TotalWins {
    let mut result = TotalWins::new();
    for _ in 0..times {
        let mut game = Game::new(strategy_min_max, strategy_random);
        let scores = game.run();

        match scores.0.cmp(&scores.1) {
            std::cmp::Ordering::Less => result.blue_wins += 1,
            std::cmp::Ordering::Equal => result.ties += 1,
            std::cmp::Ordering::Greater => result.red_wins += 1,
        }
    }
    result
}

fn main() {
    let games_per_thread = 10000;
    let threads = 10;
    let total_games = threads * games_per_thread;
    let mut result = TotalWins::new();

    let mut handles = Vec::new();
    for _ in 0..threads {
        handles.push(thread::spawn(move || -> TotalWins {
            run_games(games_per_thread)
        }));
    }

    handles.into_iter().for_each(|handle| {
        result = result.add(handle.join().unwrap());
    });

    let red_win_rate = (result.red_wins as f64 / total_games as f64) * 100f64;
    let blue_win_rate = (result.blue_wins as f64 / total_games as f64) * 100f64;
    let tie_rate = (result.ties as f64 / total_games as f64) * 100f64;
    println!(
        "red wins: {} ({}%),\nblue win: {} ({}%),\nties: {} ({}%)",
        result.red_wins, red_win_rate, result.blue_wins, blue_win_rate, result.ties, tie_rate
    );
}
