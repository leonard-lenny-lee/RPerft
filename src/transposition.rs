use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

pub enum Probe<T> {
    Read(T),
    Write,
    Ignore,
}

pub trait Entry: Sized + Clone + Copy {
    fn key(&self) -> u64;
    fn depth(&self) -> u8;
    fn age(&self) -> u8;
    fn size_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    fn new_empty() -> Self;
}

pub struct HashTable<T: Entry> {
    entries: Box<[T]>,
    size: usize,
    pub age: u8,
}

impl<T: Entry> HashTable<T> {
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / T::size_bytes();
        let vec = vec![T::new_empty(); size];
        Self {
            entries: vec.into_boxed_slice(),
            size,
            age: 1,
        }
    }

    pub fn get(&self, key: u64, depth: u8) -> Probe<T> {
        let idx = key as usize % self.size;
        let entry = self.entries[idx];
        let (entry_key, entry_depth) = (entry.key(), entry.depth());
        // Check if the entry key matches the access key
        if entry_key == key {
            // Key match so correct position. If entry depth is greater than
            // or equal to the probe depth, then return the entry
            if entry_depth >= depth {
                return Probe::Read(entry);
            }
            // Else the entry is from a lower depth search and so can be
            // replaced with a higher quality entry
            return Probe::Write;
        } else {
            // Key mismatch. If the entry is aged (from previous search), replace
            // with the newer entry from the current search; recycles old entries
            if self.age > entry.age() {
                return Probe::Write;
            }
            // Else the entry is from the current search as it is impossible for
            // entry age to be greater than the table age. In this case, keep
            // entry if the entry has a depth as it contains more information
            if entry_depth >= depth {
                return Probe::Ignore;
            }
            // Else, overwrite with a higher quality entry
            return Probe::Write;
        }
        // * Note: the empty entries which initalise the table should have a
        // * a depth and age of 0 so will always be overwritten
    }

    pub fn set(&mut self, new_entry: T) {
        let idx = new_entry.key() as usize % self.size;
        let entry = &mut self.entries[idx];
        *entry = new_entry
    }

    pub fn clear(&mut self) {
        self.entries = vec![T::new_empty(); self.size].into_boxed_slice();
    }
}

#[derive(Clone, Copy)]
pub struct PerftEntry {
    pub key: u64,
    pub count: u64,
    pub depth: u8,
    pub age: u8,
}

impl Entry for PerftEntry {
    fn key(&self) -> u64 {
        return self.key;
    }

    fn depth(&self) -> u8 {
        return self.depth;
    }

    fn age(&self) -> u8 {
        return self.age;
    }

    fn new_empty() -> Self {
        Self {
            key: 0,
            count: 0,
            depth: 0,
            age: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SearchEntry {
    pub key: u64,
    pub depth: u8,
    pub age: u8,
    pub best_move: movelist::Move,
    pub evaluation: i32,
    pub node_type: search::NodeType,
}

impl Entry for SearchEntry {
    fn key(&self) -> u64 {
        return self.key;
    }

    fn depth(&self) -> u8 {
        return self.depth;
    }

    fn age(&self) -> u8 {
        return self.age;
    }

    fn new_empty() -> Self {
        Self {
            key: 0,
            depth: 0,
            age: 0,
            best_move: movelist::Move::new_null(),
            evaluation: 0,
            node_type: search::NodeType::PV,
        }
    }
}

pub trait SharedEntry: Sized + Clone {
    fn key(&self) -> u64;
    fn data(&self) -> u64;
    fn key_ref(&self) -> &AtomicU64;
    fn data_ref(&self) -> &AtomicU64;
    fn depth(&self) -> u8 {
        // ! Always store depth as the lowest 8 bits of data
        self.data() as u8
    }
    fn size_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    fn new_empty() -> Self;
}

pub struct SharedPerftEntry {
    key: AtomicU64,
    data: AtomicU64,
}

impl Clone for SharedPerftEntry {
    fn clone(&self) -> Self {
        Self {
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

impl SharedEntry for SharedPerftEntry {
    fn key(&self) -> u64 {
        self.key.load(Ordering::Relaxed)
    }

    fn data(&self) -> u64 {
        self.data.load(Ordering::Relaxed)
    }

    fn new_empty() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    fn key_ref(&self) -> &AtomicU64 {
        &self.key
    }

    fn data_ref(&self) -> &AtomicU64 {
        &self.data
    }
}

impl SharedPerftEntry {
    /// Return the two quad words that should be stored in the hash table
    pub fn new(key: u64, depth: u8, count: u64) -> (u64, u64) {
        // Use the upper 56 bits to store the count; this is more than enough
        // The lowest 8 bits to store the depth
        let data = depth as u64 | (count << 8);
        (key ^ data, data)
    }

    pub fn count(&self) -> u64 {
        (self.data.load(Ordering::Relaxed) >> 8) as u64
    }
}

pub struct SharedHashTable<T: SharedEntry> {
    entries: Arc<[T]>,
    size: usize,
}

impl<T: SharedEntry> SharedHashTable<T> {
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / T::size_bytes();
        log::info!(
            "Hash table initialized: {:.1} Mb, {} entries ({} b/entry)",
            size_bytes as f64 / 1_000_000.0,
            size,
            T::size_bytes()
        );
        let entries = vec![T::new_empty(); size].into();
        Self { entries, size }
    }

    pub fn get(&self, key: u64, depth: u8) -> Option<T> {
        let idx = key as usize % self.size;
        let entry = &self.entries[idx];
        if entry.key() ^ entry.data() == key && entry.depth() == depth {
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn set(&self, new_entry: (u64, u64), key: u64) {
        let idx = key as usize % self.size;
        let entry = &self.entries[idx];
        entry.key_ref().store(new_entry.0, Ordering::Relaxed);
        entry.data_ref().store(new_entry.1, Ordering::Relaxed)
    }
}
