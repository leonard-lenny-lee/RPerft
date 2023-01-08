#[derive(Clone, Copy)]
pub enum SearchMethod {
    Negamax,
    AlphaBeta,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub table_size: usize,
    pub n_threads: usize,
    pub perft_config: PerftConfig,
    pub uci_mode: bool,
    pub uci_debug: bool,
    pub search_method: SearchMethod,
}

impl Config {
    pub fn initialize() -> Config {
        Config {
            table_size: 24_000_000,
            n_threads: num_cpus::get(),
            perft_config: PerftConfig::initialize(),
            uci_mode: false,
            uci_debug: false,
            search_method: SearchMethod::AlphaBeta,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PerftConfig {
    pub multithreading: bool,
    pub num_threads: usize,
    pub hashing: bool,
    pub table_size: usize,
    pub bulk_counting: bool,
}

impl PerftConfig {
    pub fn initialize() -> Self {
        Self {
            multithreading: true,
            num_threads: num_cpus::get(),
            hashing: true,
            table_size: 24_000_000,
            bulk_counting: true,
        }
    }

    pub fn report_config(&self) {
        macro_rules! report_bool {
            ($self: ident, $field: ident) => {
                if $self.$field {
                    "enabled"
                } else {
                    "disabled"
                }
            };
        }
        println!(
            "multithreading {}{}, bulk counting {}, hashing {}",
            report_bool!(self, multithreading),
            if self.multithreading {
                format!(" ({} threads)", self.num_threads)
            } else {
                "".to_string()
            },
            report_bool!(self, bulk_counting),
            report_bool!(self, hashing)
        );
    }
}
