use super::*;
use position::Position;
use config::Config;

pub struct State {
    position: Position,
    position_history: Vec<Position>,
    config: Config
}

impl State {
    pub fn initalize() -> Self {
        tables::initialize_tables(); // Initalize magic tables
        Self {
            position: Position::from_fen(common::DEFAULT_FEN.to_string()).unwrap(),
            position_history: Vec::new(),
            config: Config::initialize()
        }
    }
}