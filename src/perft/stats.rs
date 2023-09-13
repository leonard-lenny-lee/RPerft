use super::*;

use std::ops::{Add, AddAssign};
use std::time::Instant;

use prettytable::Row;

use movelist::MoveCounter;

pub struct Stats {
    start: Instant,
    pub depth: u8,
    pub count: MoveCounter,
    pub cache_stats: CacheStats,
    pub duration_sec: f64,
    pub m_nodes_per_sec: f64,
}

impl Stats {
    pub fn new(depth: u8) -> Self {
        Self {
            depth,
            start: Instant::now(),
            count: MoveCounter::default(),
            cache_stats: CacheStats::default(),
            duration_sec: 0f64,
            m_nodes_per_sec: 0f64,
        }
    }

    pub fn end(&mut self) {
        self.duration_sec = self.start.elapsed().as_secs_f64();
        self.m_nodes_per_sec = self.count.nodes as f64 / (self.duration_sec * 1_000_000.0);
    }

    pub fn start_row(cfg: &Config) -> Row {
        let mut row = row![
            br->"depth",
            br->"nodes",
            br->"sec",
            br->"Mn/s",
        ];

        if cfg.detailed {
            let detailed_headers = ["capt.", "ep", "castles", "promo."];
            for c in detailed_headers {
                row.add_cell(cell!(br->c))
            }
        }

        if cfg.caching {
            for c in [
                "accesses",
                "hits",
                "misses",
                "collisions",
                "hit nodes",
                "% cached nodes",
            ] {
                row.add_cell(cell!(br->c))
            }
        }
        row
    }

    pub fn to_row(&self, cfg: &Config) -> Row {
        let mut row = row![
            r->self.depth,
            r->self.count.nodes,
            r->format!("{:.3}", self.duration_sec),
            r->format!("{:.3}", self.m_nodes_per_sec),
        ];

        if cfg.detailed {
            let detailed_info = [
                self.count.captures,
                self.count.ep,
                self.count.castles,
                self.count.promotions,
            ];

            for info in detailed_info {
                row.add_cell(cell!(r->info))
            }
        }

        if cfg.caching {
            let cache_info = [
                self.cache_stats.accesses(),
                self.cache_stats.hits,
                self.cache_stats.misses,
                self.cache_stats.collisions,
                self.cache_stats.hit_nodes,
            ];
            for info in cache_info {
                row.add_cell(cell!(r->info))
            }
            let cache_contribution =
                self.cache_stats.hit_nodes as f64 / self.count.nodes as f64 * 100f64;
            row.add_cell(cell!(r->format!("{:.3}", cache_contribution)))
        }
        row
    }
}

#[derive(Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub collisions: u64,
    pub hit_nodes: u64,
}

impl CacheStats {
    fn accesses(&self) -> u64 {
        self.hits + self.misses + self.collisions
    }
}

impl Add for CacheStats {
    type Output = CacheStats;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            hits: self.hits + rhs.hits,
            misses: self.misses + rhs.misses,
            collisions: self.collisions + rhs.collisions,
            hit_nodes: self.hit_nodes + rhs.hit_nodes,
        }
    }
}

impl AddAssign for CacheStats {
    fn add_assign(&mut self, rhs: Self) {
        self.hits += rhs.hits;
        self.misses += rhs.misses;
        self.collisions += rhs.collisions;
        self.hit_nodes += rhs.hit_nodes;
    }
}
