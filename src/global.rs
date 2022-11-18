pub mod maps;
use super::position::Position;
use super::evaluation;

pub struct Score {
    value: i64,
}

impl Score {
    pub fn new() -> Score {
        let val: i64 = 0;
        return Score {value: val};
    }
}
pub struct State {
    position: Position,
    score: Score,
}

// This initializes the game context
impl State {

    pub fn new_from_fen(fen: Option<String>) -> State {
        let position = Position::new_from_fen(fen);
        let score = evaluation::evaluate(&position);
        return State {position, score};
    }
}