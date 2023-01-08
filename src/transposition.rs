use super::*;

pub trait Entry: Sized + Clone + Copy {
    fn key(&self) -> u64;
    fn depth(&self) -> i8;
    fn size_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    fn new_empty() -> Self;
}

pub struct TranspositionTable<T: Entry> {
    entries: Box<[T]>,
    size: usize,
}

impl<T: Entry> TranspositionTable<T> {
    pub fn new(size_bytes: usize) -> Self {
        let size = size_bytes / T::size_bytes();
        let vec = vec![T::new_empty(); size];
        Self {
            entries: vec.into_boxed_slice(),
            size,
        }
    }

    pub fn get(&self, key: u64, depth: i8) -> Option<T> {
        let idx = key as usize % self.size;
        let entry = self.entries[idx];
        if entry.key() == key && entry.depth() == depth {
            return Some(entry);
        } else {
            return None;
        }
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
    pub count: i64,
    pub depth: i8,
}

impl Entry for PerftEntry {
    fn key(&self) -> u64 {
        return self.key;
    }

    fn depth(&self) -> i8 {
        return self.depth;
    }

    fn new_empty() -> Self {
        Self {
            key: 0,
            count: 0,
            depth: -1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SearchEntry {
    pub key: u64,
    pub depth: i8,
    pub best_move: movelist::Move,
    pub evaluation: i32,
    pub node_type: search::NodeType,
}

impl Entry for SearchEntry {
    fn key(&self) -> u64 {
        return self.key;
    }

    fn depth(&self) -> i8 {
        return self.depth;
    }

    fn new_empty() -> Self {
        Self {
            key: 0,
            depth: -1,
            best_move: movelist::Move::new_null(),
            evaluation: 0,
            node_type: search::NodeType::PV,
        }
    }
}
