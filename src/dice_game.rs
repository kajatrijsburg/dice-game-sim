use std::{collections::HashMap, fmt::Display};

use rand::{thread_rng, Rng};

pub type Strategy = fn(&Board, &GameState, &usize) -> usize;
pub struct Game {
    board: Board,
    state: GameState,
    strategy_red: Strategy,
    strategy_blue: Strategy,
}

#[derive(Clone)]
pub struct Board {
    data: Vec<Option<usize>>,
    pub columns: usize,
    pub rows: usize,
}

#[derive(PartialEq)]
pub enum GameState {
    Starting,
    RedsTurn,
    BluesTurn,
    Done,
}

pub enum Team {
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

    pub fn insert(&mut self, team: &Team, column: usize, roll: usize) {
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
            if self.data[i].is_none() {
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

    pub fn has_space(&self, team: &Team, column: usize) -> bool {
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

    pub fn calculate_column_score(&self, team: &Team, column: usize) -> usize {
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

    pub fn calculate_score(&self, team: &Team) -> usize {
        let mut score = 0;
        for i in 0..self.columns {
            score += self.calculate_column_score(team, i);
        }

        score
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        write!(f, "{}", str)
    }
}

impl Game {
    pub fn new(strategy_red: Strategy, strategy_blue: Strategy) -> Self {
        Game {
            board: Board::empty_board(3, 3),
            state: GameState::Starting,
            strategy_blue,
            strategy_red,
        }
    }

    pub fn advance(&mut self) {
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
    pub fn run_and_print(&mut self) -> (usize, usize) {
        while self.state != GameState::Done {
            self.advance();
            println!("{}", self);
        }
        let scores = (
            self.board.calculate_score(&Team::Red),
            self.board.calculate_score(&Team::Blue),
        );
        println!("Red scored: {}", scores.0);
        println!("Blue scored: {}", scores.1);
        scores
    }

    pub fn run(&mut self) -> (usize, usize) {
        while self.state != GameState::Done {
            self.advance();
        }

        (
            self.board.calculate_score(&Team::Red),
            self.board.calculate_score(&Team::Blue),
        )
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();

        match self.state {
            GameState::Starting => str.push_str("The game is yet to start."),
            GameState::RedsTurn => str.push_str("Red to go."),
            GameState::BluesTurn => str.push_str("Blue to go."),
            GameState::Done => str.push_str("Game is finished"),
        }

        str.push_str("\n------\n");
        str.push_str(&self.board.to_string());

        write!(f, "{}", str)
    }
}
