use super::*;
use movelist::Move;
use search::NodeType;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

pub enum Probe<T> {
    Read(T),
    Write,
    Ignore,
}

/// Executes the logic to determine whether an entry is retrieved from the
/// table or to raise the overwrite flag
macro_rules! get_entry {
    ($self: ident, $key: ident, $depth: ident) => {
        let idx = $key as usize % $self.size;
        let entry = &$self.entries[idx];
        let (entry_key, entry_depth) = (entry.key(), entry.depth());
        // Check if the entry key matches the access key
        if entry_key == $key {
            // Key match -> entry depth >= search depth ? read : overwrite
            if entry_depth >= $depth {
                return Probe::Read(entry.clone());
            }
            return Probe::Write;
        } else {
            // Key mismatch -> entry age < table age ? overwrite : consider depth
            if $self.age >= entry.age() {
                return Probe::Write;
            }
            // entry depth >= search depth ? ignore : overwrite
            if entry_depth >= $depth {
                return Probe::Ignore;
            }
            return Probe::Write;
            // * Note: the empty entries which initalise the table should have a
            // * a depth and age of 0 so will always be overwritten
        }
    };
}

pub trait Entry: Clone {
    fn key(&self) -> u64;
    fn depth(&self) -> u8;
    fn age(&self) -> u8;
    fn new_empty() -> Self;
}

pub struct HashTable<T: Entry> {
    entries: Box<[T]>,
    size: usize,
    pub age: u8,
}

impl<T: Entry> HashTable<T> {
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / std::mem::size_of::<T>();
        let vec = vec![T::new_empty(); size];
        log::info!(
            "Hash table initialized: {:.1} Mb, {} entries ({} b/entry)",
            size_bytes as f64 / 1_000_000.0,
            size,
            T::size_bytes()
        );
        Self {
            entries: vec.into(),
            size,
            age: 1,
        }
    }

    pub fn get(&self, key: u64, depth: u8) -> Probe<T> {
        get_entry!(self, key, depth);
    }

    pub fn set(&mut self, new_entry: T) {
        let idx = new_entry.key() as usize % self.size;
        let entry = &mut self.entries[idx];
        *entry = new_entry;
    }

    pub fn clear(&mut self) {
        self.entries = vec![T::new_empty(); self.size].into();
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
    pub best_move: Move,
    pub evaluation: i16,
    pub node_type: NodeType,
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
            best_move: Move::new_null(),
            evaluation: 0,
            node_type: NodeType::PV,
        }
    }
}

pub trait SharedEntry: Clone {
    fn key(&self) -> u64;
    fn data(&self) -> u64;
    fn depth(&self) -> u8 {
        // ! Always store depth as the lowest 8 bits of data
        self.data() as u8
    }
    fn age(&self) -> u8;
    fn new_empty() -> Self;
    fn write(&self, key: u64, data: u64);
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
        self.key.load(Ordering::Relaxed) ^ self.data()
    }

    fn data(&self) -> u64 {
        self.data.load(Ordering::Relaxed)
    }

    fn age(&self) -> u8 {
        0
    }

    fn new_empty() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    fn write(&self, key: u64, data: u64) {
        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }
}

impl SharedPerftEntry {
    /// Return the two quad words that should be stored in the hash table
    pub fn encode(key: u64, depth: u8, count: u64) -> (u64, u64) {
        // Use the upper 56 bits to store the count; this is more than enough
        // The lowest 8 bits to store the depth
        let data = depth as u64 | (count << 8);
        (key ^ data, data)
    }

    pub fn count(&self) -> u64 {
        (self.data.load(Ordering::Relaxed) >> 8) as u64
    }
}

pub struct SharedSearchEntry {
    key: AtomicU64,
    data: AtomicU64,
}

impl Clone for SharedSearchEntry {
    fn clone(&self) -> Self {
        Self {
            key: AtomicU64::new(self.key.load(Ordering::Relaxed)),
            data: AtomicU64::new(self.data.load(Ordering::Relaxed)),
        }
    }
}

impl SharedEntry for SharedSearchEntry {
    fn key(&self) -> u64 {
        self.key.load(Ordering::Relaxed)
    }

    fn data(&self) -> u64 {
        self.data.load(Ordering::Relaxed)
    }

    fn age(&self) -> u8 {
        (self.data() >> 8) as u8
    }

    fn new_empty() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    fn write(&self, key: u64, data: u64) {
        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }
}

impl SharedSearchEntry {
    /*          info stored in entry
        +-------+------------+------+
        |  Bits |      Field | Type |
        +-------+------------+------+
        |   0-7 |      depth |   u8 |
        |  8-15 |        age |   u8 |
        | 16-23 |   nodetype |   u8 |
        | 24-31 | bestmove.1 |   u8 |
        | 32=39 | bestmove.2 |   u8 |
        | 40-55 |      score |  i16 |
        +-------+------------+------+
    */
    /// Retun the two quad words that should be stored in the hash table
    pub fn encode(
        key: u64,
        depth: u8,
        age: u8,
        best_move: Move,
        score: i16,
        node_type: NodeType,
    ) -> (u64, u64) {
        let data = depth as u64
            | (age as u64) << 8
            | (node_type as u64) << 16
            | (best_move.word_one() as u64) << 24
            | (best_move.word_two() as u64) << 32
            | (score as u64) << 40;
        return (key ^ data, data);
    }

    pub fn decode(&self) -> (Move, i16) {
        let data = self.data();
        return (
            Move::from_words((data >> 24) as u8, (data >> 32) as u8),
            (data >> 40) as i16,
        );
    }
}

pub struct SharedHashTable<T: SharedEntry> {
    entries: Arc<[T]>,
    size: usize,
    age: u8,
}

impl<T: SharedEntry> SharedHashTable<T> {
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / std::mem::size_of::<T>();
        log::info!(
            "Hash table initialized: {:.1} Mb, {} entries ({} b/entry)",
            size_bytes as f64 / 1_000_000.0,
            size,
            std::mem::size_of::<T>()
        );
        let entries = vec![T::new_empty(); size].into();
        Self {
            entries,
            size,
            age: 1,
        }
    }

    pub fn get(&self, key: u64, depth: u8) -> Probe<T> {
        get_entry!(self, key, depth);
    }

    pub fn set(&self, new_entry: (u64, u64), key: u64) {
        let idx = key as usize % self.size;
        let entry = &self.entries[idx];
        entry.write(new_entry.0, new_entry.1);
    }
}
