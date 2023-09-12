use crate::movelist::MoveCounter;

use std::time::Instant;

pub struct Stats {
    start: Instant,
    pub depth: u8,
    pub count: MoveCounter,
    pub duration_sec: f64,
    pub m_nodes_per_sec: f64,
}

impl Stats {
    pub fn new(depth: u8) -> Self {
        Self {
            depth,
            start: Instant::now(),
            count: MoveCounter::default(),
            duration_sec: 0f64,
            m_nodes_per_sec: 0f64,
        }
    }

    pub fn end(&mut self) {
        self.duration_sec = self.start.elapsed().as_secs_f64();
        self.m_nodes_per_sec = self.count.nodes as f64 / (self.duration_sec * 1_000_000.0);
    }

    pub fn start_row() -> prettytable::Row {
        row![
            br->"depth",
            br->"nodes",
            br->"capt.",
            br->"ep",
            br->"castles",
            br->"promo.",
            br->"sec",
            br->"Mn/s"
        ]
    }

    pub fn to_row(&self) -> prettytable::Row {
        row![
            r->self.depth,
            r->self.count.nodes,
            r->self.count.captures,
            r->self.count.ep,
            r->self.count.castles,
            r->self.count.promotions,
            r->format!("{:.3}", self.duration_sec),
            r->format!("{:.3}", self.m_nodes_per_sec),
        ]
    }
}
