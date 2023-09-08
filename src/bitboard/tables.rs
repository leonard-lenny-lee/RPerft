/// Compile time generated lookup tables.
use super::*;
use types::Axis;

macro_rules! generate_tables_array_64 {
    ($func: ident) => {{
        let mut maps = [BitBoard(0); 64];
        let mut i = 0;
        while i < 64 {
            maps[i] = $func(i);
            i += 1;
        }
        maps
    }};
}

const KNIGHT_ATTACKS: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        BitBoard(1 << sq as usize).generate_knight_attacks()
    }
    generate_tables_array_64!(f)
};

const KING_ATTACKS: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        BitBoard(1 << sq as usize).generate_king_attacks()
    }
    generate_tables_array_64!(f)
};

const RANK_TABLE: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        constants::rank::RANK_MASKS[sq / 8]
    }
    generate_tables_array_64!(f)
};

const FILE_TABLE: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        constants::file::FILE_MASKS[sq % 8]
    }
    generate_tables_array_64!(f)
};

const DIAGONAL_TABLE: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        BitBoard(BitBoard(1 << sq).ks_no_ea_fill().0 | BitBoard(1 << sq).ks_so_we_fill().0)
    }
    generate_tables_array_64!(f)
};

const ANTIDIAGIONAL_TABLE: [BitBoard; 64] = {
    const fn f(sq: usize) -> BitBoard {
        BitBoard(BitBoard(1 << sq).ks_no_we_fill().0 | BitBoard(1 << sq).ks_so_ea_fill().0)
    }
    generate_tables_array_64!(f)
};

impl BitBoard {
    #[inline(always)]
    /// Return the attack squares of a single knight by lookup
    pub fn lookup_knight_attacks(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return KNIGHT_ATTACKS[self.to_square()];
    }

    pub fn lookup_knight_attacks_(&self, _occ: BitBoard) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return KNIGHT_ATTACKS[self.to_square()];
    }

    #[inline(always)]
    /// Return the attack squares of a king by lookup
    pub fn lookup_king_attacks(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return KING_ATTACKS[self.to_square()];
    }

    #[inline(always)]
    /// Return the diagonal mask
    pub fn lookup_diagonal_mask(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return DIAGONAL_TABLE[self.to_square()];
    }

    #[inline(always)]
    /// Return the anti-diagonal mask
    pub fn lookup_antidiagonal_mask(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return ANTIDIAGIONAL_TABLE[self.to_square()];
    }

    #[inline(always)]
    /// Return the file mask
    pub fn lookup_file_mask(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return FILE_TABLE[self.to_square()];
    }

    #[inline(always)]
    /// Return the rank mask
    pub fn lookup_rank_mask(&self) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        return RANK_TABLE[self.to_square()];
    }

    #[inline(always)]
    pub fn lookup_axes_array(&self) -> [BitBoard; 4] {
        debug_assert!(self.pop_count() == 1);
        let sq = self.to_square();
        return [
            FILE_TABLE[sq],
            RANK_TABLE[sq],
            DIAGONAL_TABLE[sq],
            ANTIDIAGIONAL_TABLE[sq],
        ];
    }

    /// Use the o-2s trick to find valid squares for sliding pieces, taking
    /// into account the occupancy of the current board
    pub fn hyp_quint(&self, occ: BitBoard, axis: Axis) -> BitBoard {
        debug_assert!(self.pop_count() == 1);
        let mask = match axis {
            Axis::File => FILE_TABLE[self.to_square()],
            Axis::Rank => RANK_TABLE[self.to_square()],
            Axis::Diagonal => DIAGONAL_TABLE[self.to_square()],
            Axis::AntiDiagonal => ANTIDIAGIONAL_TABLE[self.to_square()],
        };
        let mut forward = occ & mask;
        let mut reverse = forward.reverse_bits();
        forward -= *self * 2;
        reverse -= self.reverse_bits() * 2;
        forward ^= reverse.reverse_bits();
        forward &= mask;
        forward
    }
}
