use super::*;

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

const BISHOP_MAGIC_DB: [[BB; 512]; 64] = init_bishop_magic_db();

// This is a hacky workaround the compiler const evaluation iteration limits.
// By seperating the rook data and combining them, it stops any one const fn
// being called which has an iteration count greater than the limit.

const ROOK_00: [BB; 4096] = init_rook_db(0);
const ROOK_01: [BB; 4096] = init_rook_db(1);
const ROOK_02: [BB; 4096] = init_rook_db(2);
const ROOK_03: [BB; 4096] = init_rook_db(3);
const ROOK_04: [BB; 4096] = init_rook_db(4);
const ROOK_05: [BB; 4096] = init_rook_db(5);
const ROOK_06: [BB; 4096] = init_rook_db(6);
const ROOK_07: [BB; 4096] = init_rook_db(7);
const ROOK_08: [BB; 4096] = init_rook_db(8);
const ROOK_09: [BB; 4096] = init_rook_db(9);
const ROOK_10: [BB; 4096] = init_rook_db(10);
const ROOK_11: [BB; 4096] = init_rook_db(11);
const ROOK_12: [BB; 4096] = init_rook_db(12);
const ROOK_13: [BB; 4096] = init_rook_db(13);
const ROOK_14: [BB; 4096] = init_rook_db(14);
const ROOK_15: [BB; 4096] = init_rook_db(15);
const ROOK_16: [BB; 4096] = init_rook_db(16);
const ROOK_17: [BB; 4096] = init_rook_db(17);
const ROOK_18: [BB; 4096] = init_rook_db(18);
const ROOK_19: [BB; 4096] = init_rook_db(19);
const ROOK_20: [BB; 4096] = init_rook_db(20);
const ROOK_21: [BB; 4096] = init_rook_db(21);
const ROOK_22: [BB; 4096] = init_rook_db(22);
const ROOK_23: [BB; 4096] = init_rook_db(23);
const ROOK_24: [BB; 4096] = init_rook_db(24);
const ROOK_25: [BB; 4096] = init_rook_db(25);
const ROOK_26: [BB; 4096] = init_rook_db(26);
const ROOK_27: [BB; 4096] = init_rook_db(27);
const ROOK_28: [BB; 4096] = init_rook_db(28);
const ROOK_29: [BB; 4096] = init_rook_db(29);
const ROOK_30: [BB; 4096] = init_rook_db(30);
const ROOK_31: [BB; 4096] = init_rook_db(31);
const ROOK_32: [BB; 4096] = init_rook_db(32);
const ROOK_33: [BB; 4096] = init_rook_db(33);
const ROOK_34: [BB; 4096] = init_rook_db(34);
const ROOK_35: [BB; 4096] = init_rook_db(35);
const ROOK_36: [BB; 4096] = init_rook_db(36);
const ROOK_37: [BB; 4096] = init_rook_db(37);
const ROOK_38: [BB; 4096] = init_rook_db(38);
const ROOK_39: [BB; 4096] = init_rook_db(39);
const ROOK_40: [BB; 4096] = init_rook_db(40);
const ROOK_41: [BB; 4096] = init_rook_db(41);
const ROOK_42: [BB; 4096] = init_rook_db(42);
const ROOK_43: [BB; 4096] = init_rook_db(43);
const ROOK_44: [BB; 4096] = init_rook_db(44);
const ROOK_45: [BB; 4096] = init_rook_db(45);
const ROOK_46: [BB; 4096] = init_rook_db(46);
const ROOK_47: [BB; 4096] = init_rook_db(47);
const ROOK_48: [BB; 4096] = init_rook_db(48);
const ROOK_49: [BB; 4096] = init_rook_db(49);
const ROOK_50: [BB; 4096] = init_rook_db(50);
const ROOK_51: [BB; 4096] = init_rook_db(51);
const ROOK_52: [BB; 4096] = init_rook_db(52);
const ROOK_53: [BB; 4096] = init_rook_db(53);
const ROOK_54: [BB; 4096] = init_rook_db(54);
const ROOK_55: [BB; 4096] = init_rook_db(55);
const ROOK_56: [BB; 4096] = init_rook_db(56);
const ROOK_57: [BB; 4096] = init_rook_db(57);
const ROOK_58: [BB; 4096] = init_rook_db(58);
const ROOK_59: [BB; 4096] = init_rook_db(59);
const ROOK_60: [BB; 4096] = init_rook_db(60);
const ROOK_61: [BB; 4096] = init_rook_db(61);
const ROOK_62: [BB; 4096] = init_rook_db(62);
const ROOK_63: [BB; 4096] = init_rook_db(63);

const ROOK_MAGIC_DB: [&[BB; 4096]; 64] = [
    &ROOK_00, &ROOK_01, &ROOK_02, &ROOK_03, &ROOK_04, &ROOK_05, &ROOK_06, &ROOK_07,
    &ROOK_08, &ROOK_09, &ROOK_10, &ROOK_11, &ROOK_12, &ROOK_13, &ROOK_14, &ROOK_15,
    &ROOK_16, &ROOK_17, &ROOK_18, &ROOK_19, &ROOK_20, &ROOK_21, &ROOK_22, &ROOK_23,
    &ROOK_24, &ROOK_25, &ROOK_26, &ROOK_27, &ROOK_28, &ROOK_29, &ROOK_30, &ROOK_31,
    &ROOK_32, &ROOK_33, &ROOK_34, &ROOK_35, &ROOK_36, &ROOK_37, &ROOK_38, &ROOK_39,
    &ROOK_40, &ROOK_41, &ROOK_42, &ROOK_43, &ROOK_44, &ROOK_45, &ROOK_46, &ROOK_47,
    &ROOK_48, &ROOK_49, &ROOK_50, &ROOK_51, &ROOK_52, &ROOK_53, &ROOK_54, &ROOK_55,
    &ROOK_56, &ROOK_57, &ROOK_58, &ROOK_59, &ROOK_60, &ROOK_61, &ROOK_62, &ROOK_63,
];

const fn init_bishop_magic_db() -> [[BB; 512]; 64] {

    let mut db = [[BB(0); 512]; 64];

    let mut square = 0;
    while square < 64 {
        let magic = BISHOP_MAGICS[square];
        let n_bits_in_mask = BISHOP_MASKS[square].count_ones();
        let n_entries = 1 << n_bits_in_mask;
        let mut square_db = [BB(0); 512];
        let mut db_idx = 0;
        while db_idx < n_entries {
            let mut mask = BISHOP_MASKS[square];
            let mut occ = 0u64;
            let mut mask_idx = 0;
            while mask_idx < n_bits_in_mask {
                let bit = 1 << mask.trailing_zeros();
                mask ^= bit;
                if (1 << mask_idx) & db_idx != 0 {
                    occ |= bit
                }
                mask_idx += 1;
            }
            let key = (occ.wrapping_mul(magic) >> BISHOP_SHIFTS[square]) as usize;
            square_db[key] = BB(1 << square).const_bishop_attacks_hyp_quint(occ);
            db_idx += 1;
        }
        db[square] = square_db;
        square += 1;
    }
    db
}

const fn init_rook_db(square: usize) -> [BB; 4096] {

    let magic = ROOK_MAGICS[square];
    let n_bits_in_mask = ROOK_MASKS[square].count_ones();
    let n_entries = 1 << n_bits_in_mask;
    let mut square_db = [BB(0); 4096];
    let mut db_idx = 0;
    while db_idx < n_entries {
        let mut mask = ROOK_MASKS[square];
        let mut occ = 0u64;
        let mut mask_idx = 0;
        while mask_idx < n_bits_in_mask {
            let bit = 1 << mask.trailing_zeros();
            mask ^= bit;
            if (1 << mask_idx) & db_idx != 0 {
                occ |= bit
            }
            mask_idx += 1;
        }
        let key = (occ.wrapping_mul(magic) >> ROOK_SHIFTS[square]) as usize;
        square_db[key] = BB(1 << square).const_rook_attacks_hyp_quint(occ);
        db_idx += 1;
    }
    square_db
}

impl BB {

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
    
    const fn const_hyp_quint(&self, occ: u64, axis: Axis) -> u64 {
        let idx = self.0.trailing_zeros() as usize;
        let mask = match axis {
            Axis::File => MAPS.file_masks[idx],
            Axis::Rank => MAPS.rank_masks[idx],
            Axis::Diagonal => MAPS.diag_masks[idx],
            Axis::AntiDiagonal => MAPS.adiag_masks[idx]
        };
        let mut forward = occ & mask.0;
        let mut reverse = forward.reverse_bits();
        forward = forward.wrapping_sub(self.0.wrapping_mul(2));
        reverse = reverse.wrapping_sub(self.0.reverse_bits().wrapping_mul(2));
        forward ^= reverse.reverse_bits();
        forward &= mask.0;
        return forward
    }

    const fn const_rook_attacks_hyp_quint(&self, occ: u64) -> BB {
        BB(self.const_hyp_quint(occ, Axis::File) | self.const_hyp_quint(occ, Axis::Rank)) 
    }

    const fn const_bishop_attacks_hyp_quint(&self, occ: u64) -> BB {
        BB(self.const_hyp_quint(occ, Axis::Diagonal) | self.const_hyp_quint(occ, Axis::AntiDiagonal))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

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