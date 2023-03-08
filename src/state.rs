use super::*;
use config::Config;
use position::Position;
use transposition::HashTable;

pub struct State {
    pub position: Position,
    pub position_history: Vec<Position>,
    pub config: Config,
    pub transposition_table: HashTable,
}

impl State {
    pub fn initalize() -> Self {
        tables::initialize(); // Initalize magic tables
        let config = Config::initialize();
        Self {
            position: Position::from_fen(common::DEFAULT_FEN.to_string()).unwrap(),
            position_history: Vec::new(),
            transposition_table: HashTable::new(config.table_size),
            config,
        }
    }
}
