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
    pub fn initialize() -> Config {
        Config {
            table_size: 24_000_000,
            num_threads: num_cpus::get(),
            uci_mode: false,
            uci_debug: false,
            search_method: SearchMethod::AlphaBeta,
        }
    }
}
