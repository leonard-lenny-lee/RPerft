use std::time::Instant;

pub struct Stats<'a> {
    fen: &'a str,
    depth: u8,
    cache_size: usize,
    num_threads: usize,
    start: Instant,
    pub node_count: u64,
    pub duration_sec: f64,
    pub m_nodes_per_sec: f64,
}

impl<'a> Stats<'a> {
    pub fn new(fen: &'a str, depth: u8, cache_size: usize, num_threads: usize) -> Self {
        Self {
            fen,
            depth,
            cache_size,
            num_threads,
            start: Instant::now(),
            node_count: 0,
            duration_sec: 0f64,
            m_nodes_per_sec: 0f64,
        }
    }

    pub fn end(&mut self, node_count: u64) {
        self.duration_sec = self.start.elapsed().as_secs_f64();
        self.node_count = node_count;
        self.m_nodes_per_sec = node_count as f64 / (self.duration_sec * 1_000_000.0);
    }

    pub fn report(&self) {
        todo!()
    }
}
