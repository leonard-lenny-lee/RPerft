/// Caching for transposition
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Cache {
    entries: Box<[Entry]>,
    size: usize,
}

impl Cache {
    /// Initialize
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / std::mem::size_of::<Entry>();
        let vec = vec![Entry::new_empty(); size];
        log::info!(
            "Cache initialized: {:.1} Mb, {} entries ({} b/entry)",
            size_bytes as f64 / 1_000_000.0,
            size,
            std::mem::size_of::<Entry>()
        );
        return Self {
            entries: vec.into(),
            size,
        };
    }

    /// Retrieve the stored node count from the table, if it exists. Else, return None.
    pub fn fetch(&self, key: u64, depth: u8) -> Option<u64> {
        let index = key as usize % self.size;

        let entry = unsafe { self.entries.get_unchecked(index) };
        let entry_key = entry.key();
        if key == entry_key {
            let (entry_depth, node_count) = entry.decode();
            if depth == entry_depth {
                return Some(node_count);
            }
        }
        return None;
    }

    /// Write a perft entry into the hash table
    pub fn store(&self, key: u64, depth: u8, node_count: u64) {
        let idx = key as usize % self.size;
        let entry = unsafe { self.entries.get_unchecked(idx) };
        entry.encode(key, depth, node_count)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries = vec![Entry::new_empty(); self.size].into();
    }
}

// Cache entries are stored as atomic quad words to ensure thread safety
struct Entry {
    key: AtomicU64,
    val: AtomicU64,
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            val: AtomicU64::new(self.val.load(Ordering::Relaxed)),
        }
    }
}

impl Entry {
    fn key(&self) -> u64 {
        return self.key.load(Ordering::Relaxed) ^ self.val();
    }

    fn val(&self) -> u64 {
        return self.val.load(Ordering::Relaxed);
    }

    fn write(&self, key: u64, val: u64) {
        self.key.store(key ^ val, Ordering::Relaxed);
        self.val.store(val, Ordering::Relaxed);
    }

    // Store depth and node count
    fn encode(&self, key: u64, depth: u8, count: u64) {
        // Upper 56 bits store node count, lowest 8 bits store depth
        let val = depth as u64 | (count << 8);
        self.write(key, val);
    }

    // Return depth and node count as a tuple
    fn decode(&self) -> (u8, u64) {
        let data = self.val();
        return (data as u8, data >> 8);
    }

    fn new_empty() -> Self {
        return Self {
            key: AtomicU64::new(0),
            val: AtomicU64::new(0),
        };
    }
}
