use core::panic;
use std::{collections::HashMap, thread};

use rand::{thread_rng, Rng};

type Strategy = fn(&Board, &GameState, &usize) -> usize;
struct Game {
    board: Board,
    state: GameState,
    strategy_red: Strategy,
    strategy_blue: Strategy,
}

#[derive(Clone)]
struct Board {
    data: Vec<Option<usize>>,
    pub columns: usize,
    pub rows: usize,
}

#[derive(PartialEq)]
enum GameState {
    Starting,
    RedsTurn,
    BluesTurn,
    Done,
}

enum Team {
    Red,
    Blue,
}

impl Board {
    fn empty_board(columns: usize, rows: usize) -> Self {
        Board {
            data: vec![None; rows * columns * 2],
            columns,
            rows,
        }
    }

    fn get_board_half(&self, team: &Team) -> &[Option<usize>] {
        let split_point = self.rows * self.columns;
        match team {
            Team::Red => &self.data[..split_point],
            Team::Blue => &self.data[split_point..],
        }
    }

    fn get_column(&self, team: &Team, column: usize) -> &[Option<usize>] {
        assert!(column < self.columns, "request collumn does not exist.");
        let board_half = self.get_board_half(team);
        let offset = column * self.rows;

        &board_half[offset..offset + self.rows]
    }

    fn insert(&mut self, team: &Team, column: usize, roll: usize) {
        assert!(column < self.columns);

        let team_offset;
        let opponent_team_offset;
        match team {
            Team::Red => {
                team_offset = 0;
                opponent_team_offset = self.columns * self.rows;
            }
            Team::Blue => {
                team_offset = self.columns * self.rows;
                opponent_team_offset = 0;
            }
        }

        let start_index = team_offset + column * self.rows;
        let end_index = start_index + self.rows;

        let mut insertion_index = None;
        for i in start_index..end_index {
            if self.data[i] == None {
                insertion_index = Some(i);
            }
        }

        match insertion_index {
            Some(index) => self.data[index] = Some(roll),
            None => {
                panic!("Collumn does not have a free space!")
            }
        }

        let start_index = opponent_team_offset + column * self.rows;
        let end_index = start_index + self.rows;

        for i in start_index..end_index {
            if self.data[i] == Some(roll) {
                self.data[i] = None;
            }
        }
    }

    fn to_string(&self) -> String {
        let mut str = String::new();
        let no_val = '-';
        let seperator = '-';
        let board_seperator = "------\n";

        for (i, option) in self.data.iter().enumerate() {
            match option {
                Some(num) => {
                    str.push_str(&num.to_string());
                }
                None => {
                    str.push(no_val);
                }
            }

            str.push(seperator);

            if i == 0 {
                continue;
            }

            if i % self.rows == 0 {
                str.push('\n');
            }
            if i % self.rows * self.columns == 0 {
                str.push_str(board_seperator);
            }
        }

        str
    }

    fn has_space(&self, team: &Team, column: usize) -> bool {
        assert!(column < self.columns);
        let col = self.get_column(team, column);

        for option in col {
            if option.is_none() {
                return true;
            }
        }
        false
    }

    fn is_board_side_filled(&self, team: &Team) -> bool {
        for i in 0..self.columns {
            if self.has_space(team, i) {
                return false;
            }
        }

        true
    }

    fn calculate_column_score(&self, team: &Team, column: usize) -> usize {
        let col = self.get_column(team, column);

        let mut rolls: HashMap<usize, usize> = HashMap::new();
        for option in col {
            match option {
                Some(num) => {
                    if rolls.contains_key(num) {
                        let val = rolls.get(num).unwrap() + 1;
                        rolls.insert(*num, val);
                    } else {
                        rolls.insert(*num, 1);
                    }
                }
                None => (),
            }
        }
        let mut score = 0;
        for entry in rolls {
            score += entry.0 * entry.1 * entry.1;
        }
        score
    }

    fn calculate_score(&self, team: &Team) -> usize {
        let mut score = 0;
        for i in 0..self.columns {
            score += self.calculate_column_score(team, i);
        }

        score
    }
}

impl Game {
    fn new(strategy_red: Strategy, strategy_blue: Strategy) -> Self {
        Game {
            board: Board::empty_board(3, 3),
            state: GameState::Starting,
            strategy_blue,
            strategy_red,
        }
    }

    fn to_string(&self) -> String {
        let mut str = String::new();

        match self.state {
            GameState::Starting => str.push_str("The game is yet to start."),
            GameState::RedsTurn => str.push_str("Red to go."),
            GameState::BluesTurn => str.push_str("Blue to go."),
            GameState::Done => str.push_str("Game is finished"),
        }

        str.push_str("\n------\n");
        str.push_str(&self.board.to_string());

        str
    }

    fn advance(&mut self) {
        match self.state {
            GameState::Starting => {
                let coin_toss: bool = thread_rng().gen();
                match coin_toss {
                    true => self.state = GameState::RedsTurn,
                    false => self.state = GameState::BluesTurn,
                };
            }

            GameState::RedsTurn => {
                let roll = thread_rng().gen_range(1..=6);
                let col = (self.strategy_red)(&self.board, &self.state, &roll);
                self.board.insert(&Team::Red, col, roll);

                match self.board.is_board_side_filled(&Team::Red) {
                    true => self.state = GameState::Done,
                    false => self.state = GameState::BluesTurn,
                }
            }
            GameState::BluesTurn => {
                let roll = thread_rng().gen_range(1..=6);
                let col = (self.strategy_blue)(&self.board, &self.state, &roll);
                self.board.insert(&Team::Blue, col, roll);

                match self.board.is_board_side_filled(&Team::Blue) {
                    true => self.state = GameState::Done,
                    false => self.state = GameState::RedsTurn,
                }
            }
            GameState::Done => (),
        }
    }

    #[allow(dead_code)]
    fn run_and_print(&mut self) -> (usize, usize) {
        while self.state != GameState::Done {
            self.advance();
            println!("{}", self.to_string());
        }
        let scores = (
            self.board.calculate_score(&Team::Red),
            self.board.calculate_score(&Team::Blue),
        );
        println!("Red scored: {}", scores.0);
        println!("Blue scored: {}", scores.1);
        scores
    }

    fn run(&mut self) -> (usize, usize) {
        while self.state != GameState::Done {
            self.advance();
        }

        return (
            self.board.calculate_score(&Team::Red),
            self.board.calculate_score(&Team::Blue),
        );
    }
}

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
    let team;
    match state {
        GameState::RedsTurn => team = Team::Red,
        GameState::BluesTurn => team = Team::Blue,
        _ => panic!("Cannot take a turn in this game state."),
    }

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

        if scores.0 > scores.1 {
            result.red_wins += 1;
        } else if scores.0 < scores.1 {
            result.blue_wins += 1;
        } else {
            result.ties += 1;
        }
    }
    result
}

fn main() {
    let games_per_thread = 10000;
    let threads = 1;
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
