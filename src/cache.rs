/// Caching for transposition
use super::*;

use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};

use movelist::*;

pub enum Access {
    Hit(MoveCounter),
    Miss,
    Collision,
}

pub struct Cache<T: SizedEntry> {
    entries: Box<[T]>,
    size: usize,
}

impl<T: SizedEntry> Cache<T> {
    /// Initialize cache
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / size_of::<T>();
        let vec = vec![T::default(); size];
        return Self {
            entries: vec.into(),
            size,
        };
    }

    /// Retrieve stored count information from the cache
    pub fn read(&self, key: u64, depth: u8) -> Access {
        let index = key as usize % self.size;
        let entry = unsafe { self.entries.get_unchecked(index) };

        if key == entry.key() {
            let (entry_depth, count) = entry.load();
            if depth == entry_depth {
                return Access::Hit(count);
            }
            return Access::Collision;
        }
        return Access::Miss;
    }

    /// Write a perft entry into the cache
    pub fn write(&self, key: u64, depth: u8, count: &MoveCounter) {
        let index = key as usize % self.size;
        unsafe { self.entries.get_unchecked(index).store(key, depth, count) };
    }
}

pub trait SizedEntry: Entry + Sized + Clone + Default + Sync + Send {}

pub trait Entry {
    fn key(&self) -> u64;
    // Return depth and count info as a tuple
    fn load(&self) -> (u8, MoveCounter);
    // Store depth and count info
    fn store(&self, key: u64, depth: u8, count: &MoveCounter);
}

#[derive(Default)]
pub struct Entry2xU64 {
    wordq_0: AtomicU64,
    wordq_1: AtomicU64,
}

impl Clone for Entry2xU64 {
    fn clone(&self) -> Self {
        Self {
            wordq_0: AtomicU64::new(self.wordq_0.load(Ordering::Relaxed)),
            wordq_1: AtomicU64::new(self.wordq_1.load(Ordering::Relaxed)),
        }
    }
}

impl Entry for Entry2xU64 {
    fn key(&self) -> u64 {
        self.wordq_0() ^ self.wordq_1()
    }

    fn load(&self) -> (u8, MoveCounter) {
        let wordq_1 = self.wordq_1();
        let count = MoveCounter {
            nodes: wordq_1 >> 8,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
        };
        return (wordq_1 as u8, count);
    }

    fn store(&self, key: u64, depth: u8, count: &MoveCounter) {
        let wordq_1 = depth as u64 | count.nodes << 8;
        let wordq_0 = key ^ wordq_1;

        self.wordq_0.store(wordq_0, Ordering::Relaxed);
        self.wordq_1.store(wordq_1, Ordering::Relaxed);
    }
}

impl Entry2xU64 {
    fn wordq_0(&self) -> u64 {
        self.wordq_0.load(Ordering::Relaxed)
    }

    fn wordq_1(&self) -> u64 {
        self.wordq_1.load(Ordering::Relaxed)
    }
}

impl SizedEntry for Entry2xU64 {}

#[derive(Default)]
pub struct Entry4xU64 {
    wordq_0: AtomicU64,
    wordq_1: AtomicU64, // Depth and node count
    wordq_2: AtomicU64, // Captures and ep
    wordq_3: AtomicU64, // Castles and promotions
}

impl Clone for Entry4xU64 {
    fn clone(&self) -> Self {
        Self {
            wordq_0: AtomicU64::new(self.wordq_0.load(Ordering::Relaxed)),
            wordq_1: AtomicU64::new(self.wordq_1.load(Ordering::Relaxed)),
            wordq_2: AtomicU64::new(self.wordq_2.load(Ordering::Relaxed)),
            wordq_3: AtomicU64::new(self.wordq_3.load(Ordering::Relaxed)),
        }
    }
}

impl Entry for Entry4xU64 {
    fn key(&self) -> u64 {
        self.wordq_0() ^ self.wordq_1() ^ self.wordq_2() ^ self.wordq_3()
    }

    fn load(&self) -> (u8, MoveCounter) {
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

    fn store(&self, key: u64, depth: u8, count: &MoveCounter) {
        // Upper 56 bits store node count, lowest 8 bits store depth
        let wordq_1 = depth as u64 | count.nodes << 8;
        let wordq_2 = (count.captures as u64) << 32 | count.ep as u64;
        let wordq_3 = (count.castles as u64) << 32 | count.promotions as u64;
        let wordq_0 = key ^ wordq_1 ^ wordq_2 ^ wordq_3;

        self.wordq_0.store(wordq_0, Ordering::Relaxed);
        self.wordq_1.store(wordq_1, Ordering::Relaxed);
        self.wordq_2.store(wordq_2, Ordering::Relaxed);
        self.wordq_3.store(wordq_3, Ordering::Relaxed);
    }
}

impl Entry4xU64 {
    fn wordq_0(&self) -> u64 {
        self.wordq_0.load(Ordering::Relaxed)
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
}

impl SizedEntry for Entry4xU64 {}
