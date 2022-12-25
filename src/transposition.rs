#[derive(Clone, Copy)]
pub struct PerftEntry {
    key: u64,
    pub count: i64,
    depth: i8
    // Currently 17 bytes per entry
}

pub struct PerftTable {
    entries: Box<[PerftEntry]>,
    size: usize // Number of entries
}

impl PerftTable {

    pub fn new(size_bytes: usize) -> PerftTable {
        // Calculate the number of entries possible and initialise
        let size = size_bytes / 17;
        let vec = vec![
            PerftEntry{
                key: 0,
                count: 0,
                depth: -1
            };
            size
        ];
        PerftTable {
            entries: vec.into_boxed_slice(),
            size: size
        }
    }

    pub fn get(&mut self, key: u64, depth: i8) -> Option<PerftEntry> {
        let idx = key as usize % self.size;
        let entry = self.entries[idx];
        if entry.key == key && entry.depth == depth {
            Some(entry)
        } else {
            None
        }
    }

    pub fn set(&mut self, key: u64, count: i64, depth: i8) {
        let idx = key as usize % self.size;
        let entry = &mut self.entries[idx];
        *entry = PerftEntry {key, count, depth}
    }

}