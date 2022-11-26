pub mod maps;
use super::position::Data;
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
    position: Data,
    evaluation: i32,
}

// This initializes the game context
impl State {

    pub fn new_from_fen(fen: String) -> State {
        let position = Data::from_fen(fen);
        let score = evaluation::evaluate(&position);
        let evaluation = 0;
        return State {position, evaluation};
    }
}