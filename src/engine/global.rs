// Default configuration

const HASHING_ENABLED: bool = true;
const TABLE_SIZE: usize = 17_000_000;

pub struct Global {
    pub hashing_enabled: bool,
    pub table_size: usize,
}

impl Global {
    pub const fn init() -> Global {
        Global {
            hashing_enabled: HASHING_ENABLED,
            table_size: TABLE_SIZE
        }
    }

    pub fn report_config(&self) {
        println!("Hashing: {}", self.hashing_enabled)
    }
}
