/// Module contains an implementation of magic bitboards
use super::*;

lazy_static! {
    static ref ROOK_ATTACKS: MagicTable = MagicTable::new(TableType::Rook);
    static ref BISHOP_ATTACKS: MagicTable = MagicTable::new(TableType::Bishop);
    static ref BETWEEN_BB: [[BB; 64]; 64] = {
        let mut tables = [[BB(0); 64]; 64];
        for sq_1 in 0..64 {
            for sq_2 in 0..64 {
                tables[sq_1][sq_2] = BB::from_sq(sq_1).between(BB::from_sq(sq_2));
            }
        }
        tables
    };
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
    pext_enabled: bool,
}

impl MagicTable {
    fn new(table_type: TableType) -> Self {
        let pext_enabled;
        #[cfg(target_arch = "x86_64")]
        {
            pext_enabled = is_x86_feature_detected!("bmi2")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            pext_enabled = false
        }
        let mut table = match table_type {
            TableType::Rook => Self {
                tables: vec![vec![BB(0); 4096]; 64],
                table_type,
                magic_factors: &keys::ROOK_MAGICS,
                masks: &keys::ROOK_MASKS,
                shifts: &keys::ROOK_SHIFTS,
                pext_enabled,
            },
            TableType::Bishop => Self {
                tables: vec![vec![BB(0); 512]; 64],
                table_type,
                magic_factors: &keys::BISHOP_MAGICS,
                masks: &keys::BISHOP_MASKS,
                shifts: &keys::BISHOP_SHIFTS,
                pext_enabled,
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
                let key = if self.pext_enabled {
                    unsafe { std::arch::x86_64::_pext_u64(occ, *mask) as usize }
                } else {
                    (occ.wrapping_mul(*magic) >> shift) as usize
                };
                self.tables[sq][key] = match self.table_type {
                    TableType::Bishop => BB::from_sq(sq).bishop_hq(BB(occ)),
                    TableType::Rook => BB::from_sq(sq).rook_hq(BB(occ)),
                };
            }
        }
    }

    fn lu(&self, sq: BB, occ: BB) -> BB {
        assert!(sq.0.count_ones() == 1);
        let sq_key = sq.ils1b();
        // Hash the occlusion bitboard
        let key = if self.pext_enabled {
            unsafe { std::arch::x86_64::_pext_u64(occ.0, self.masks[sq_key]) }
        } else {
            (occ.0 & self.masks[sq_key]).wrapping_mul(self.magic_factors[sq_key])
                >> self.shifts[sq_key]
        };
        return self.tables[sq_key][key as usize];
    }
}

impl BB {
    /// Find the rook attack squares by looking up the magic tables
    pub fn rook_lu(&self, occ: BB) -> BB {
        return ROOK_ATTACKS.lu(*self, occ);
    }

    /// Find the bishop attack squares by looking up the magic tables
    pub fn bishop_lu(&self, occ: BB) -> BB {
        return BISHOP_ATTACKS.lu(*self, occ);
    }

    /// Find the queen attack squares by lookup up the magic tables
    pub fn queen_lu(&self, occ: BB) -> BB {
        self.rook_lu(occ) | self.bishop_lu(occ)
    }

    fn between(&self, bb_2: BB) -> BB {
        assert_eq!(self.0.count_ones(), 1);
        assert_eq!(bb_2.0.count_ones(), 1);
        return bb_2 |
            // On same file or rank
            if ((self.file() | self.rank()) & bb_2).is_not_empty() {
                self.rook_lu(bb_2) & bb_2.rook_lu(*self)
            }
            // On same diagonal or antidiagonal
            else if ((self.diag() | self.adiag()) & bb_2).is_not_empty() {
                self.bishop_lu(bb_2) & bb_2.bishop_lu(*self)
            }
            // else, don't share an axis
            else {
                EMPTY_BB
            };
    }

    /// Return a bitboard with the intervening bits between this single bit
    /// bitboard and another single bit bitboard filled. If they do not share
    /// a common axis, return the other bitboard.
    pub fn between_bb(&self, bb_2: BB) -> BB {
        let (sq_1, sq_2) = (self.to_sq(), bb_2.to_sq());
        return BETWEEN_BB[sq_1][sq_2];
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
            TableType::Bishop => (keys::BISHOP_MAGICS, keys::BISHOP_MASKS, keys::BISHOP_SHIFTS),
            TableType::Rook => (keys::ROOK_MAGICS, keys::ROOK_MASKS, keys::ROOK_SHIFTS),
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