/// Implementation of magic bitboards
use super::*;

lazy_static! {
    static ref ROOK_ATTACKS: MagicTable = MagicTable::new(TableType::Rook);
    static ref BISHOP_ATTACKS: MagicTable = MagicTable::new(TableType::Bishop);
    static ref BETWEEN_TABLES: [[BitBoard; 64]; 64] = {
        let mut tables = [[BitBoard(0); 64]; 64];
        for sq_1 in 0..64 {
            for sq_2 in 0..64 {
                tables[sq_1][sq_2] =
                    BitBoard::from_square(sq_1).between(BitBoard::from_square(sq_2));
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
    tables: Box<[[BitBoard; 4096]; 64]>,
    table_type: TableType,
    magic_factors: &'static [u64; 64],
    masks: &'static [u64; 64],
    shifts: &'static [u64; 64],
}

impl MagicTable {
    fn new(table_type: TableType) -> Self {
        let mut table = match table_type {
            TableType::Rook => Self {
                tables: Box::new([[BitBoard(0); 4096]; 64]),
                table_type,
                magic_factors: &keys::ROOK_MAGICS,
                masks: &keys::ROOK_MASKS,
                shifts: &keys::ROOK_SHIFTS,
            },
            TableType::Bishop => Self {
                tables: Box::new([[BitBoard(0); 4096]; 64]),
                table_type,
                magic_factors: &keys::BISHOP_MAGICS,
                masks: &keys::BISHOP_MASKS,
                shifts: &keys::BISHOP_SHIFTS,
            },
        };
        table.init();
        return table;
    }

    fn init(&mut self) {
        use std::iter::zip;

        for (sq, ((magic_factor, mask), shift)) in
            zip(zip(self.magic_factors, self.masks), self.shifts).enumerate()
        {
            let n_bits = mask.count_ones();
            let n_permutations = 1 << n_bits;
            // Enumerate through all the possible combinations of 0 and 1
            // for a given mask, with 2 ** n_bits possible combinations
            for sq_table_idx in 0..n_permutations {
                // Build occupancy mask
                let mut mask_copy = *mask;
                let mut occ = 0u64;
                for mask_idx in 0..n_bits {
                    let bit = 1 << mask_copy.trailing_zeros();
                    mask_copy ^= bit;
                    if (1 << mask_idx) & sq_table_idx != 0 {
                        occ |= bit
                    }
                }
                // Hash the occupancy config and use it to store the
                // attacks in that config, as calculated by hyp quint
                let key;
                #[cfg(USE_PEXT)]
                {
                    key = unsafe { std::arch::x86_64::_pext_u64(occ, *mask) as usize }
                };
                #[cfg(not(USE_PEXT))]
                {
                    key = (occ.wrapping_mul(*magic_factor) >> shift) as usize
                };
                self.tables[sq][key] = match self.table_type {
                    TableType::Bishop => {
                        BitBoard::from_square(sq).hyp_quint_bishop_attacks(BitBoard(occ))
                    }
                    TableType::Rook => {
                        BitBoard::from_square(sq).hyp_quint_rook_attacks(BitBoard(occ))
                    }
                };
            }
        }
    }

    #[cfg(USE_PEXT)]
    fn lu(&self, sq: BitBoard, occ: BitBoard) -> BitBoard {
        let sq_key = sq.ils1b();
        let key = unsafe { std::arch::x86_64::_pext_u64(occ.0, self.masks[sq_key]) };
        self.tables[sq_key][key as usize]
    }

    #[cfg(not(USE_PEXT))]
    fn lu(&self, sq: BitBoard, occ: BitBoard) -> BitBoard {
        assert!(sq.0.count_ones() == 1);
        let sq_key = sq.get_ls1b_index();
        let key = (occ.0 & self.masks[sq_key]).wrapping_mul(self.magic_factors[sq_key])
            >> self.shifts[sq_key];
        self.tables[sq_key][key as usize]
    }
}

impl BitBoard {
    /// Find the rook attack squares by looking up the magic tables
    pub fn magic_rook_attacks(&self, occ: BitBoard) -> BitBoard {
        return ROOK_ATTACKS.lu(*self, occ);
    }

    /// Find the bishop attack squares by looking up the magic tables
    pub fn magic_bishop_attacks(&self, occ: BitBoard) -> BitBoard {
        return BISHOP_ATTACKS.lu(*self, occ);
    }

    /// Find the queen attack squares by lookup up the magic tables
    pub fn magic_queen_attacks(&self, occ: BitBoard) -> BitBoard {
        self.magic_rook_attacks(occ) | self.magic_bishop_attacks(occ)
    }

    fn between(&self, bb_2: BitBoard) -> BitBoard {
        assert_eq!(self.0.count_ones(), 1);
        assert_eq!(bb_2.0.count_ones(), 1);
        return bb_2 |
            // On same file or rank
            if ((self.lookup_file_mask() | self.lookup_rank_mask()) & bb_2).is_not_empty() {
                self.magic_rook_attacks(bb_2) & bb_2.magic_rook_attacks(*self)
            }
            // On same diagonal or antidiagonal
            else if ((self.lookup_diagonal_mask() | self.lookup_antidiagonal_mask()) & bb_2).is_not_empty() {
                self.magic_bishop_attacks(bb_2) & bb_2.magic_bishop_attacks(*self)
            }
            // else, don't share an axis
            else {
                constants::bb::EMPTY
            };
    }

    /// Return a bitboard with the intervening bits between this single bit
    /// bitboard and another single bit bitboard filled. If they do not share
    /// a common axis, return the other bitboard.
    pub fn between_bb(&self, bb_2: BitBoard) -> BitBoard {
        let (sq_1, sq_2) = (self.to_square(), bb_2.to_square());
        return BETWEEN_TABLES[sq_1][sq_2];
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
