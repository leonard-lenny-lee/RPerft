use super::*;

/// Caching for transposition
use std::sync::atomic::{AtomicU64, Ordering};

use movelist::*;

pub struct Cache {
    entries: Box<[Entry]>,
    size: usize,
}

impl Cache {
    /// Initialize
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / std::mem::size_of::<Entry>();
        let vec = vec![Entry::default(); size];
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
    pub fn fetch(&self, key: u64, depth: u8) -> Option<MoveCounter> {
        let index = key as usize % self.size;

        let entry = unsafe { self.entries.get_unchecked(index) };
        let entry_key = entry.key();
        if key == entry_key {
            let (entry_depth, count) = entry.decode();
            if depth == entry_depth {
                return Some(count);
            }
        }
        return None;
    }

    /// Write a perft entry into the hash table
    pub fn store(&self, key: u64, depth: u8, count: &MoveCounter) {
        let idx = key as usize % self.size;
        let entry = unsafe { self.entries.get_unchecked(idx) };
        entry.encode(key, depth, count)
    }

    /// Clear all entries
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries = vec![Entry::default(); self.size].into();
    }
}

// Cache entries are stored as atomic quad words to ensure thread safety
#[derive(Default)]
struct Entry {
    key: AtomicU64,
    wordq_1: AtomicU64, // Depth and node count
    wordq_2: AtomicU64, // Captures and ep
    wordq_3: AtomicU64, // Castles and promotions
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            wordq_1: AtomicU64::new(self.wordq_1.load(Ordering::Relaxed)),
            wordq_2: AtomicU64::new(self.wordq_2.load(Ordering::Relaxed)),
            wordq_3: AtomicU64::new(self.wordq_3.load(Ordering::Relaxed)),
        }
    }
}

impl Entry {
    fn key(&self) -> u64 {
        self.key.load(Ordering::Relaxed) ^ self.wordq_1() ^ self.wordq_2() ^ self.wordq_3()
    }

    fn wordq_1(&self) -> u64 {
        self.wordq_1.load(Ordering::Relaxed)
    }

    fn wordq_2(&self) -> u64 {
        self.wordq_2.load(Ordering::Relaxed)
    }

    fn wordq_3(&self) -> u64 {
        self.wordq_3.load(Ordering::Relaxed)
    }

    fn write(&self, key: u64, wordq_1: u64, wordq_2: u64, wordq_3: u64) {
        let key = key ^ wordq_1 ^ wordq_2 ^ wordq_3;
        self.key.store(key, Ordering::Relaxed);
        self.wordq_1.store(wordq_1, Ordering::Relaxed);
        self.wordq_2.store(wordq_2, Ordering::Relaxed);
        self.wordq_3.store(wordq_3, Ordering::Relaxed);
    }

    // Store depth and count info
    fn encode(&self, key: u64, depth: u8, count: &MoveCounter) {
        // Upper 56 bits store node count, lowest 8 bits store depth
        let wordq_1 = depth as u64 | count.nodes << 8;
        let wordq_2 = (count.captures as u64) << 32 | count.ep as u64;
        let wordq_3 = (count.castles as u64) << 32 | count.promotions as u64;
        self.write(key, wordq_1, wordq_2, wordq_3);
    }

    // Return depth and count info as a tuple
    fn decode(&self) -> (u8, MoveCounter) {
        let wordq_1 = self.wordq_1();
        let wordq_2 = self.wordq_2();
        let wordq_3 = self.wordq_3();
        let count = MoveCounter {
            nodes: wordq_1 >> 8,
            captures: (wordq_2 >> 32) as u32,
            ep: wordq_2 as u32,
            castles: (wordq_3 >> 32) as u32,
            promotions: wordq_3 as u32,
        };
        return (wordq_1 as u8, count);
    }
}
