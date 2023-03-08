/// Module contains an implementation of magic bitboards
use super::*;

lazy_static! {
    static ref ROOK_ATTACKS: MagicTable = MagicTable::new(TableType::Rook);
    static ref BISHOP_ATTACKS: MagicTable = MagicTable::new(TableType::Bishop);
}

/// Initialise lazily evaluated static tables at program startup
pub fn initialize() {
    ROOK_ATTACKS.lu(BB(1), BB(0));
    BISHOP_ATTACKS.lu(BB(1), BB(0));
}

enum TableType {
    Rook,
    Bishop,
}

struct MagicTable {
    tables: Vec<Vec<BB>>,
    table_type: TableType,
    magic_factors: &'static [u64; 64],
    masks: &'static [u64; 64],
    shifts: &'static [u64; 64],
}

impl MagicTable {
    fn new(table_type: TableType) -> Self {
        let mut table = match table_type {
            TableType::Rook => Self {
                tables: vec![vec![BB(0); 4096]; 64],
                table_type,
                magic_factors: &constants::ROOK_MAGICS,
                masks: &constants::ROOK_MASKS,
                shifts: &constants::ROOK_SHIFTS,
            },
            TableType::Bishop => Self {
                tables: vec![vec![BB(0); 512]; 64],
                table_type,
                magic_factors: &constants::BISHOP_MAGICS,
                masks: &constants::BISHOP_MASKS,
                shifts: &constants::BISHOP_SHIFTS,
            },
        };
        table.init_tables();
        return table;
    }

    fn init_tables(&mut self) {
        use std::iter::zip;
        for (sq, ((magic, mask), shift)) in
            zip(zip(self.magic_factors, self.masks), self.shifts).enumerate()
        {
            let n_bits = mask.count_ones();
            let n_permutations = 1 << n_bits;
            // Enumerate through all the possible combinations of 0 and 1
            // for a given mask, with 2 ** n_bits possible combinations
            for sq_table_idx in 0..n_permutations {
                // Build occupancy mask
                let mut m = *mask;
                let mut occ = 0u64;
                for mask_idx in 0..n_bits {
                    let bit = 1 << m.trailing_zeros();
                    m ^= bit;
                    if (1 << mask_idx) & sq_table_idx != 0 {
                        occ |= bit
                    }
                }
                // Hash the occupancy config and use it to store the
                // attacks in that config, as calculated by hyp quint
                let key = (occ.wrapping_mul(*magic) >> shift) as usize;
                self.tables[sq][key] = match self.table_type {
                    TableType::Bishop => BB::from_index(sq).bishop_hq(BB(occ)),
                    TableType::Rook => BB::from_index(sq).rook_hq(BB(occ)),
                };
            }
        }
    }

    fn lu(&self, sq: BB, occ: BB) -> BB {
        assert!(sq.0.count_ones() == 1);
        let sq_key = sq.ils1b();
        // Hash the occlusion bitboard
        let key = (occ.0 & self.masks[sq_key]).wrapping_mul(self.magic_factors[sq_key])
            >> self.shifts[sq_key];
        return self.tables[sq_key][key as usize];
    }
}

impl BB {
    /// Find the rook attack squares by looking up the magic tables
    pub fn lu_rook_attacks(&self, occ: BB) -> BB {
        return ROOK_ATTACKS.lu(*self, occ);
    }

    /// Find the bishop attack squares by looking up the magic tables
    pub fn lu_bishop_attacks(&self, occ: BB) -> BB {
        return BISHOP_ATTACKS.lu(*self, occ);
    }

    /// Find the queen attack squares by lookup up the magic tables
    pub fn lu_queen_attacks(&self, occ: BB) -> BB {
        self.lu_rook_attacks(occ) | self.lu_bishop_attacks(occ)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

    /// Test that magic factor hashing is free from collisions.
    #[test_case(TableType::Bishop; "bishop")]
    #[test_case(TableType::Rook; "rook")]
    fn test_magics(table_type: TableType) {
        let (magics, masks, shifts) = match table_type {
            TableType::Bishop => (
                constants::BISHOP_MAGICS,
                constants::BISHOP_MASKS,
                constants::BISHOP_SHIFTS,
            ),
            TableType::Rook => (
                constants::ROOK_MAGICS,
                constants::ROOK_MASKS,
                constants::ROOK_SHIFTS,
            ),
        };
        for square in 0..64 {
            let magic = magics[square];
            let n_bits_in_mask = masks[square].count_ones();
            let size_of_db = 1 << n_bits_in_mask;
            let mut db = vec![false; size_of_db];
            let mut failure = false;
            for db_idx in 0..size_of_db {
                let mut mask = masks[square];
                let mut occ: u64 = 0;
                for mask_idx in 0..n_bits_in_mask {
                    let bit = 1 << mask.trailing_zeros();
                    if (1 << mask_idx) & db_idx != 0 {
                        occ |= bit
                    }
                    mask ^= bit;
                }
                let key = (occ.wrapping_mul(magic) >> shifts[square]) as usize;
                if !db[key] {
                    db[key] = true;
                } else {
                    // Has already been set
                    // print!("FAILURE SQUARE {} AT {} \n", square, db_idx);
                    failure = true;
                    break;
                }
            }
            // if !failure{print!("SUCCESSFUL TEST {}\n", square)}
            assert!(!failure)
        }
    }
}
