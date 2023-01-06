pub enum SearchMethod {
    Negamax,
    AlphaBeta,
}

pub struct Config {
    pub hashing: bool,
    pub table_size: usize,
    pub bulk_counting: bool,
    pub uci_mode: bool,
    pub uci_debug: bool,
    pub search_method: SearchMethod,
}

impl Config {
    pub const fn initialize() -> Config {
        Config {
            /// Default perft configuration
            hashing: true,
            table_size: 17_000_000, // 1 million Perft entries
            bulk_counting: true,
            uci_mode: false,
            uci_debug: false,
            search_method: SearchMethod::Negamax,
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
        macro_rules! report_method {
            ($self: ident, $field: ident) => {
                match $self.$field {
                    SearchMethod::Negamax => "Negamax",
                    SearchMethod::AlphaBeta => "Alpha Beta"
                }
            };
        }
        println!(
            "bulk counting {}, hashing {}, search method {}",
            report_bool!(self, bulk_counting),
            report_bool!(self, hashing),
            report_method!(self, search_method)
        );
    }
}
