/// Configuration struct
const DEFAULT_TABLE_SIZE_BYTES: usize = 24_000_000;

#[derive(Clone, Copy)]
pub enum SearchMethod {
    Negamax,
    AlphaBeta,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub table_size: usize,
    pub num_threads: usize,
    pub uci_mode: bool,
    pub uci_debug: bool,
    pub search_method: SearchMethod,
}

impl Config {
    pub fn init() -> Config {
        let num_threads = num_cpus::get();
        log::info!(
            "hash table size {} bytes. {} cpus detected",
            DEFAULT_TABLE_SIZE_BYTES,
            num_threads
        );
        Config {
            table_size: DEFAULT_TABLE_SIZE_BYTES,
            num_threads,
            uci_mode: false,
            uci_debug: false,
            search_method: SearchMethod::AlphaBeta,
        }
    }
}
