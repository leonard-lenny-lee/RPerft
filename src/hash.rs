/// Hash Table implementation for transpositions
use std::sync::atomic::{AtomicU64, Ordering};

pub struct HashTable {
    entries: Box<[Entry]>,
    size: usize,
    pub age: u8,
}

impl HashTable {
    /// Initialize a new hash table
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / std::mem::size_of::<Entry>();
        let vec = vec![Entry::new_empty(); size];
        log::info!(
            "Hash table initialized: {:.1} Mb, {} entries ({} b/entry)",
            size_bytes as f64 / 1_000_000.0,
            size,
            std::mem::size_of::<Entry>()
        );
        return Self {
            entries: vec.into(),
            size,
            age: 1,
        };
    }

    /// Retrieve the stored node count from the table, if it exists. Else, return None.
    pub fn fetch(&self, key: u64, depth: u8) -> Option<u64> {
        let index = key as usize % self.size;
        let entry = &self.entries[index];
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
        let entry = &self.entries[idx];
        entry.encode(key, depth, node_count)
    }

    /// Clear all hash entries
    pub fn clear(&mut self) {
        self.entries = vec![Entry::new_empty(); self.size].into();
    }
}

// Table entries are stored as atomic quad words to ensure thread safety
struct Entry {
    key: AtomicU64,
    data: AtomicU64,
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

impl Entry {
    fn key(&self) -> u64 {
        return self.key.load(Ordering::Relaxed) ^ self.data();
    }

    fn data(&self) -> u64 {
        return self.data.load(Ordering::Relaxed);
    }

    fn write(&self, key: u64, data: u64) {
        self.key.store(key ^ data, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    // Store depth and node count
    fn encode(&self, key: u64, depth: u8, count: u64) {
        // Upper 56 bits store node count, lowest 8 bits store depth
        let data = depth as u64 | (count << 8);
        self.write(key, data);
    }

    // Return depth and node count as a tuple
    fn decode(&self) -> (u8, u64) {
        let data = self.data();
        return (data as u8, data >> 8);
    }

    fn new_empty() -> Self {
        return Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        };
    }
}
