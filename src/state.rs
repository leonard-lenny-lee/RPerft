use super::*;
use config::Config;
use hash::HashTable;
use position::Position;

pub struct State {
    pub position: Position,
    pub position_history: Vec<Position>,
    pub config: Config,
    pub hash_table: HashTable,
}

impl State {
    pub fn init() -> Self {
        tables::initialize(); // Initalize magic tables
        let config = Config::init();
        Self {
            position: Position::from_fen(common::DEFAULT_FEN.to_string()).unwrap(),
            position_history: Vec::new(),
            hash_table: HashTable::new(config.table_size),
            config,
        }
    }
}
