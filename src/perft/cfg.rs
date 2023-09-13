use super::*;

use prettytable::Table;

pub struct Config {
    pub multithreading: bool,
    pub caching: bool,
    pub num_threads: usize,
    pub cache_size: usize,
    pub detailed: bool,
}

impl Config {
    pub fn new(multithreading: bool, cache_size: usize, detailed: bool) -> Self {
        Self {
            multithreading,
            caching: cache_size > 0,
            num_threads: if multithreading { num_cpus::get() } else { 1 },
            cache_size,
            detailed,
        }
    }

    pub fn report(&self) -> Table {
        let mut table = Table::new();
        table.add_row(row![b->"feature", "enabled", "info"]);

        let m = if self.multithreading {
            format!("{} cores", self.num_threads)
        } else {
            format!("-")
        };

        let c = if self.caching {
            let cache_size_mb = self.cache_size as f64 / 1_000_000.0;
            let n_entries = self.cache_size / 32;
            format!("{:.2} Mb; {} entries", cache_size_mb, n_entries)
        } else {
            format!("-")
        };

        table.add_row(row![b->"multithreading", self.multithreading, m]);
        table.add_row(row![b->"cache", self.caching, c]);
        table.add_row(row![b->"detailed count", self.detailed]);
        table
    }

    pub fn test_cfg() -> Self {
        Self {
            multithreading: true,
            caching: true,
            num_threads: num_cpus::get(),
            cache_size: constants::DEFAULT_CACHE_SIZE,
            detailed: false,
        }
    }
}
