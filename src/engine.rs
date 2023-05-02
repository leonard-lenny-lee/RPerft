/// Holds the master struct representing the its current state
use super::*;
use hash::HashTable;
use nnue::NNUE;
use position::Position;

pub const DEF_TABLE_SIZE_BYTES: usize = 32_000_000;

pub struct Engine {
    pub cur_pos: Position,
    pub hash_table: HashTable,
    pub nnue: NNUE,
    pub num_threads: usize,
    pub table_size_bytes: usize,
    pub mode: EngineMode,
    pub debug: bool,
}

impl Engine {
    pub fn init() -> Self {
        Self {
            cur_pos: Position::new_starting_pos(),
            hash_table: HashTable::new(DEF_TABLE_SIZE_BYTES),
            nnue: NNUE::init(NNUE_PATH),
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
