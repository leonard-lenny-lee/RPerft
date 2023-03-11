/// Holds the master struct representing the its current state
use super::*;
use hash::HashTable;
use position::Position;

const DEF_TABLE_SIZE_BYTES: usize = 32_000_000;

pub struct Engine {
    pub cur_pos: Position,
    pub hash_table: HashTable,
    pub num_threads: usize,
    pub table_size_bytes: usize,
    pub mode: EngineMode,
    pub debug: bool,
}

impl Engine {
    pub fn init() -> Self {
        Self {
            cur_pos: Position::new_starting(),
            hash_table: HashTable::new(DEF_TABLE_SIZE_BYTES),
            num_threads: num_cpus::get(),
            table_size_bytes: DEF_TABLE_SIZE_BYTES,
            mode: EngineMode::User,
            debug: false,
        }
    }
}
// In user mode, the engine should provide verbose log messages if there
// are errors with the input. In UCI mode, input should be correct, ignore
// any messages that cannot be parsed or executed.
pub enum EngineMode {
    User,
    Uci,
}
