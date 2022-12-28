pub struct Config {
    pub hashing: bool,
    pub table_size: usize,
    pub bulk_counting: bool,
}

impl Config {
    pub const fn init() -> Config {
        Config {
            /// Default perft configuration
            hashing: true,
            table_size: 17_000_000, // 1 million Perft entries
            bulk_counting: true,
        }
    }

    pub fn report_config(&self) {
        macro_rules! report_bool {
            ($self: ident, $field: ident) => {
                if $self.$field {"enabled"} else {"disabled"}
            };
        }
        println!(
            "bulk counting {}, hashing {}",
            report_bool!(self, bulk_counting), report_bool!(self, hashing)
        );
    }
}
