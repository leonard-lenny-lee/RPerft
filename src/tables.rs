/// Contains the tables generated at compile time for fast lookups at runtime

use super::*;

const KNIGHT_ATTACK_TABLE: [BB; 64] = {
    let mut maps: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        maps[i] = BB(1 << i).knight_attack_squares();
        i += 1;
    }
    maps
};

const KING_ATTACK_TABLE: [BB; 64] = {
    let mut maps: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        maps[i] = BB(1 << i).king_attack_squares();
        i += 1;
    }
    maps
};

const RANK_TABLE: [BB; 64] = {
    let mut masks: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = RANK_MASKS[i / 8];
        i += 1;
    }
    masks
};

const FILE_TABLE: [BB; 64] = {
    let mut masks: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = FILE_MASKS[i % 8];
        i += 1;
    }
    masks
};

const DIAG_TABLE: [BB; 64] = {
    let mut masks: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        let origin = BB(1 << i);
        masks[i] = BB(origin.no_ea_fill().0 | origin.so_we_fill().0);
        i += 1;
    }
    masks
};

const ADIAG_TABLE: [BB; 64] = {
    let mut masks: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        let origin = BB(1 << i);
        masks[i] = BB(origin.no_we_fill().0 | origin.so_ea_fill().0);
        i += 1
    }
    masks
};

const WPAWN_CAPTURE_TABLE: [BB; 64] = {
    let mut table: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        let origin = BB(1 << i);
        table[i] = BB(
            ((origin.0 & !FILE_H.0) << 9) | ((origin.0 & !FILE_A.0) << 7)
        );
        i += 1;
    }
    table
};

const BPAWN_CAPTURE_TABLE: [BB; 64] = {
    let mut masks: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        let origin = BB(1 << i);
        masks[i] = BB(
            ((origin.0 & !FILE_H.0) >> 7) | ((origin.0 & !FILE_A.0) >> 9)
        );
        i += 1;
    }
    masks
};

const BISHOP_SHIFTS: [u64; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58,
	59, 59, 59, 59, 59, 59, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 59, 59, 59, 59, 59, 59,
	58, 59, 59, 59, 59, 59, 59, 58
];

const BISHOP_MAGICS: [u64; 64] = [
    0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
	0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
	0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
	0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
	0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
	0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
	0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
	0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
	0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
	0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
	0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
	0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
	0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
	0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
	0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
	0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200
];

const BISHOP_MASKS: [u64; 64] = [
    0x0040201008040200, 0x0000402010080400, 0x0000004020100A00, 0x0000000040221400,
    0x0000000002442800, 0x0000000204085000, 0x0000020408102000, 0x0002040810204000,
    0x0020100804020000, 0x0040201008040000, 0x00004020100A0000, 0x0000004022140000,
    0x0000000244280000, 0x0000020408500000, 0x0002040810200000, 0x0004081020400000,
    0x0010080402000200, 0x0020100804000400, 0x004020100A000A00, 0x0000402214001400,
    0x0000024428002800, 0x0002040850005000, 0x0004081020002000, 0x0008102040004000,
    0x0008040200020400, 0x0010080400040800, 0x0020100A000A1000, 0x0040221400142200,
    0x0002442800284400, 0x0004085000500800, 0x0008102000201000, 0x0010204000402000,
    0x0004020002040800, 0x0008040004081000, 0x00100A000A102000, 0x0022140014224000,
    0x0044280028440200, 0x0008500050080400, 0x0010200020100800, 0x0020400040201000,
    0x0002000204081000, 0x0004000408102000, 0x000A000A10204000, 0x0014001422400000,
    0x0028002844020000, 0x0050005008040200, 0x0020002010080400, 0x0040004020100800,
    0x0000020408102000, 0x0000040810204000, 0x00000A1020400000, 0x0000142240000000,
    0x0000284402000000, 0x0000500804020000, 0x0000201008040200, 0x0000402010080400,
    0x0002040810204000, 0x0004081020400000, 0x000A102040000000, 0x0014224000000000,
    0x0028440200000000, 0x0050080402000000, 0x0020100804020000, 0x0040201008040200
];


const ROOK_SHIFTS: [u64; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	52, 53, 53, 53, 53, 53, 53, 52
];

const ROOK_MAGICS: [u64; 64] = [
    0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
    0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
    0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
    0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
    0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
    0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
    0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
    0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
    0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
    0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
    0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
    0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
    0x0000102040800101, 0x0000102040008101, 0x0000081020004101, 0x0000040810002101,
    0x0001000204080011, 0x0001000204000801, 0x0001000082000401, 0x0000002040810402
];

const ROOK_MASKS: [u64; 64] = [
    0x000101010101017E, 0x000202020202027C, 0x000404040404047A, 0x0008080808080876,
    0x001010101010106E, 0x002020202020205E, 0x004040404040403E, 0x008080808080807E,
    0x0001010101017E00, 0x0002020202027C00, 0x0004040404047A00, 0x0008080808087600,
    0x0010101010106E00, 0x0020202020205E00, 0x0040404040403E00, 0x0080808080807E00,
    0x00010101017E0100, 0x00020202027C0200, 0x00040404047A0400, 0x0008080808760800,
    0x00101010106E1000, 0x00202020205E2000, 0x00404040403E4000, 0x00808080807E8000,
    0x000101017E010100, 0x000202027C020200, 0x000404047A040400, 0x0008080876080800,
    0x001010106E101000, 0x002020205E202000, 0x004040403E404000, 0x008080807E808000,
    0x0001017E01010100, 0x0002027C02020200, 0x0004047A04040400, 0x0008087608080800,
    0x0010106E10101000, 0x0020205E20202000, 0x0040403E40404000, 0x0080807E80808000,
    0x00017E0101010100, 0x00027C0202020200, 0x00047A0404040400, 0x0008760808080800,
    0x00106E1010101000, 0x00205E2020202000, 0x00403E4040404000, 0x00807E8080808000,
    0x007E010101010100, 0x007C020202020200, 0x007A040404040400, 0x0076080808080800,
    0x006E101010101000, 0x005E202020202000, 0x003E404040404000, 0x007E808080808000,
    0x7E01010101010100, 0x7C02020202020200, 0x7A04040404040400, 0x7608080808080800,
    0x6E10101010101000, 0x5E20202020202000, 0x3E40404040404000, 0x7E80808080808000
];

lazy_static! {
    static ref ROOK_MAGIC_DB: Vec<Vec<BB>> = initialize_magic_table(true);
    static ref BISHOP_MAGIC_DB: Vec<Vec<BB>> = initialize_magic_table(false);
}

/// Initialise lazily evaluated static tables at program startup
pub fn initialize_tables() {
    ROOK_MAGIC_DB[0][0];
    BISHOP_MAGIC_DB[0][0];
}

fn initialize_magic_table(rook: bool) -> Vec<Vec<BB>> {

    let (magics, masks, shifts) = if rook {
        (ROOK_MAGICS, ROOK_MASKS, ROOK_SHIFTS)
    } else {
        (BISHOP_MAGICS, BISHOP_MASKS, BISHOP_SHIFTS)
    };

    let mut db = Vec::new();
    for sq in 0..64 {
        let magic = magics[sq];
        let n_bits_in_mask = masks[sq].count_ones();
        let n_entries = 1 << n_bits_in_mask;
        let mut sq_db = vec![BB(0); n_entries];
        // Enumerate through all the possible combinations of 0 and 1
        // for a given mask, with 2 ** n_bits possible combinations 
        for db_idx in 0..n_entries {
            // Build the occupancy mask
            let mut mask = masks[sq];
            let mut occ = 0u64;
            for mask_idx in 0..n_bits_in_mask {
                let bit = 1 << mask.trailing_zeros();
                mask ^= bit;
                if (1 << mask_idx) & db_idx != 0 {
                    occ |= bit
                }
            }
            // Hash the occupancy config and use it to store the
            // attacks in that config, as calculated by hyp quint
            let key = (occ.wrapping_mul(magic) >> shifts[sq]) as usize;
            sq_db[key] = if rook {
                BB(1 << sq).rook_attacks_hyp_quint(BB(occ))
            } else {
                BB(1 << sq).bishop_attacks_hyp_quint(BB(occ))
            };
        }
        db.push(sq_db)
    };
    return db
}

impl BB {

    #[inline(always)]
    /// Return the attack squares of a single knight by lookup
    pub fn lookup_knight_attacks(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        KNIGHT_ATTACK_TABLE[self.to_index()]
    }

    #[inline(always)]
    /// Return the attack squares of a king by lookup
    pub fn lookup_king_attacks(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        KING_ATTACK_TABLE[self.to_index()]
    }

    #[inline(always)]
    /// Return the diagonal mask
    pub fn lookup_diagonal_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        DIAG_TABLE[self.to_index()]
    }

    #[inline(always)]
    /// Return the anti-diagonal mask
    pub fn lookup_anti_diagonal_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        ADIAG_TABLE[self.to_index()]
    }

    #[inline(always)]
    /// Return the file mask
    pub fn lookup_file_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        FILE_TABLE[self.to_index()]
    }

    #[inline(always)]
    /// Return the rank mask
    pub fn lookup_rank_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        RANK_TABLE[self.to_index()]
    }

    /// Return the white pawn capture mask
    pub fn lookup_wpawn_capture_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        WPAWN_CAPTURE_TABLE[self.to_index()]
    }

    /// Return the black pawn capture mask
    pub fn lookup_bpawn_capture_mask(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        BPAWN_CAPTURE_TABLE[self.to_index()]
    }

    /// Use the o-2s trick to find valid squares for sliding pieces, taking
    /// into account the occupancy of the current board
    pub fn hyp_quint(&self, occ: BB, axis: Axis) -> BB {
        debug_assert!(self.pop_count() == 1);
        let mask = match axis {
            Axis::File => FILE_TABLE[self.ils1b()],
            Axis::Rank => RANK_TABLE[self.ils1b()],
            Axis::Diagonal => DIAG_TABLE[self.ils1b()],
            Axis::AntiDiagonal => ADIAG_TABLE[self.ils1b()]
        };
        let mut forward = occ & mask;
        let mut reverse = forward.reverse_bits();
        forward -= *self * 2;
        reverse -= self.reverse_bits() * 2;
        forward ^= reverse.reverse_bits();
        forward &= mask;
        forward
    }

    /// Find the rook attack squares by looking up the magic tables
    pub fn lookup_rook_attacks(&self, occ: BB) -> BB {
        let sq = self.ils1b();
        // Hash the occulusion bitboard
        let idx = (occ.0 & ROOK_MASKS[sq]).wrapping_mul(ROOK_MAGICS[sq]) >> ROOK_SHIFTS[sq];
        ROOK_MAGIC_DB[sq][idx as usize]
    }

    /// Find the bishop attack squares by looking up the magic tables
    pub fn lookup_bishop_attacks(&self, occ: BB) -> BB {
        let sq = self.ils1b();
        let idx = (occ.0 & BISHOP_MASKS[sq]).wrapping_mul(BISHOP_MAGICS[sq]) >> BISHOP_SHIFTS[sq];
        BISHOP_MAGIC_DB[sq][idx as usize]
    }

    /// Find the queen attack squares by lookup up the magic tables
    pub fn lookup_queen_attacks(&self, occ: BB) -> BB {
        self.lookup_rook_attacks(occ) | self.lookup_bishop_attacks(occ)
    }
    
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

    /// Test that magic factor hashing is free from collisions.
    #[test_case(BISHOP_MAGICS, BISHOP_MASKS, BISHOP_SHIFTS; "bishop")]
    #[test_case(ROOK_MAGICS, ROOK_MASKS, ROOK_SHIFTS; "rook")]
    fn test_magics(magics: [u64; 64], masks: [u64; 64], shifts: [u64; 64]) {
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