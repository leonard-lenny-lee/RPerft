/// Holds the master struct representing the its current state
use super::*;
use hash::HashTable;
use position::Position;

pub const DEF_TABLE_SIZE_BYTES: usize = 32_000_000;

pub struct Engine {
    pub current_position: Position,
    pub hash_table: HashTable,
    pub num_threads: usize,
    pub table_size_bytes: usize,
    pub mode: EngineMode,
    pub debug: bool,
}

impl Engine {
    pub fn init() -> Self {
        Self {
            current_position: Position::new_starting_position(),
            hash_table: HashTable::new(DEF_TABLE_SIZE_BYTES),
            num_threads: num_cpus::get(),
            table_size_bytes: DEF_TABLE_SIZE_BYTES,
            mode: EngineMode::User,
            debug: false,
        }
    }
}

pub enum EngineMode {
    User,
    Uci,
}
