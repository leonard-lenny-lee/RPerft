/// Hash Table implementation for transpositions
use super::*;
use movelist::Move;
use search::NodeType;
use std::sync::atomic::{AtomicU64, Ordering};

pub enum Probe {
    Read(EntryData),
    Write,
    Ignore,
}

pub struct EntryData {
    depth: u8,
    age: u8,
    pub node_type: NodeType,
    pub best_move: Move,
    pub score: i16,
}

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

    /// Retrieve the stored node count from the table, if it exists. Else,
    /// return None.
    pub fn probe_perft(&self, key: u64, depth: u8) -> Option<u64> {
        let index = key as usize % self.size;
        let entry = &self.entries[index];
        let entry_key = entry.key();
        if key == entry_key {
            let (entry_depth, node_count) = entry.decode_perft();
            if depth == entry_depth {
                return Some(node_count);
            }
        }
        return None;
    }

    /// Execute algorithm to determine whether an entry is read from the table
    /// or overwritten
    pub fn probe_search(&self, key: u64, depth: u8) -> Probe {
        let index = key as usize % self.size;
        let entry = &self.entries[index];
        let (entry_key, entry_data) = (entry.key(), entry.decode_search());
        if entry_key == key {
            // *KEY MATCH
            if entry_data.depth >= depth {
                // Read from higher depth searches
                return Probe::Read(entry_data);
            }
            // Else, overwrite old entry
            return Probe::Write;
        } else {
            // *KEY MISMATCH
            if self.age >= entry_data.age {
                // Overwrite old entries
                return Probe::Write;
            }
            // Else
            if entry_data.depth >= depth {
                // Keep searches at higher depth
                return Probe::Ignore;
            }
            // Overwrite newer searches if it comes from a lower depth
            return Probe::Write;
        }
    }

    /// Write a perft entry into the hash table
    pub fn write_perft(&self, key: u64, depth: u8, node_count: u64) {
        let idx = key as usize % self.size;
        let entry = &self.entries[idx];
        entry.encode_perft(key, depth, node_count)
    }

    /// Write a search entry into the hash table
    pub fn write_search(
        &self,
        key: u64,
        depth: u8,
        best_move: Move,
        score: i16,
        node_type: NodeType,
    ) {
        let idx = key as usize % self.size;
        let entry = &self.entries[idx];
        entry.encode_search(key, depth, self.age, best_move, score, node_type);
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

    // Only depth and node count needs to be stored for perft entries
    fn encode_perft(&self, key: u64, depth: u8, count: u64) {
        // Upper 56 bits store node count, lowest 8 bits store depth
        let data = depth as u64 | (count << 8);
        self.write(key, data);
    }

    fn decode_perft(&self) -> (u8, u64) {
        let data = self.data();
        return (data as u8, data >> 8);
    }

    // Encode and store a search entry according to the scheme
    fn encode_search(
        &self,
        key: u64,
        depth: u8,
        age: u8,
        best_move: Move,
        score: i16,
        node_type: NodeType,
    ) {
        /*
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
        let data = depth as u64
            | (age as u64) << 8
            | (node_type as u64) << 16
            | (best_move.word_one() as u64) << 24
            | (best_move.word_two() as u64) << 32
            | (score as u64) << 40;
        self.write(key, data);
    }

    fn decode_search(&self) -> EntryData {
        let data = self.data();
        return EntryData {
            depth: data as u8,
            age: (data >> 8) as u8,
            node_type: NodeType::from_u8((data >> 16) as u8),
            best_move: Move::from_words((data >> 24) as u8, (data >> 32) as u8),
            score: (data >> 40) as i16,
        };
    }

    fn new_empty() -> Self {
        return Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        };
    }
}

impl NodeType {
    // Use only to load from HashTable entry
    fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::PV,
            1 => Self::Cut,
            _ => Self::All,
        }
    }
}
