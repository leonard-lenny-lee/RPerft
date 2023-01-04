use super::*;
use config::Config;
use position::Position;

pub struct State {
    pub position: Position,
    pub position_history: Vec<Position>,
    pub config: Config,
}

impl State {
    pub fn initalize() -> Self {
        tables::initialize_tables(); // Initalize magic tables
        Self {
            position: Position::from_fen(common::DEFAULT_FEN.to_string()).unwrap(),
            position_history: Vec::new(),
            config: Config::initialize(),
        }
    }
}
