// Default configuration

const HASHING_ENABLED: bool = true;
const TABLE_SIZE: u64 = 17_000_000;

pub struct Global {
    pub hashing_enabled: bool,
    pub table_size: u64,
}

impl Global {
    pub fn init() -> Global {
        Global {
            hashing_enabled: HASHING_ENABLED,
            table_size: TABLE_SIZE
        }
    }
}
