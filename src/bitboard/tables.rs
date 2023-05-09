/// Compile time generated lookup tables.
use super::*;
use types::Axis;

macro_rules! generate_table64 {
    ($f: ident) => {{
        let mut maps = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            maps[i] = $f(i);
            i += 1;
        }
        maps
    }};
}

const KNIGHT_ATTACKS: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        BB(1 << sq as usize).knight_attacks()
    }
    generate_table64!(f)
};

const KING_ATTACKS: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        BB(1 << sq as usize).king_attacks()
    }
    generate_table64!(f)
};

const RANK_TABLE: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        RANK_MASKS[sq / 8]
    }
    generate_table64!(f)
};

const FILE_TABLE: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        FILE_MASKS[sq % 8]
    }
    generate_table64!(f)
};

const DIAG_TABLE: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        BB(BB(1 << sq).no_ea_fill().0 | BB(1 << sq).so_we_fill().0)
    }
    generate_table64!(f)
};

const ADIAG_TABLE: [BB; 64] = {
    const fn f(sq: usize) -> BB {
        BB(BB(1 << sq).no_we_fill().0 | BB(1 << sq).so_ea_fill().0)
    }
    generate_table64!(f)
};

impl BB {
    #[inline(always)]
    /// Return the attack squares of a single knight by lookup
    pub fn knight_lu(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return KNIGHT_ATTACKS[self.to_sq()];
    }

    pub fn knight_lu_(&self, _occ: BB) -> BB {
        debug_assert!(self.pop_count() == 1);
        return KNIGHT_ATTACKS[self.to_sq()];
    }

    #[inline(always)]
    /// Return the attack squares of a king by lookup
    pub fn king_lu(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return KING_ATTACKS[self.to_sq()];
    }

    #[inline(always)]
    /// Return the diagonal mask
    pub fn diag(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return DIAG_TABLE[self.to_sq()];
    }

    #[inline(always)]
    /// Return the anti-diagonal mask
    pub fn adiag(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return ADIAG_TABLE[self.to_sq()];
    }

    #[inline(always)]
    /// Return the file mask
    pub fn file(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return FILE_TABLE[self.to_sq()];
    }

    #[inline(always)]
    /// Return the rank mask
    pub fn rank(&self) -> BB {
        debug_assert!(self.pop_count() == 1);
        return RANK_TABLE[self.to_sq()];
    }

    #[inline(always)]
    pub fn axes_lu(&self) -> [BB; 4] {
        debug_assert!(self.pop_count() == 1);
        let sq = self.to_sq();
        return [
            FILE_TABLE[sq],
            RANK_TABLE[sq],
            DIAG_TABLE[sq],
            ADIAG_TABLE[sq],
        ];
    }

    /// Use the o-2s trick to find valid squares for sliding pieces, taking
    /// into account the occupancy of the current board
    pub fn hyp_quint(&self, occ: BB, axis: Axis) -> BB {
        debug_assert!(self.pop_count() == 1);
        let mask = match axis {
            Axis::File => FILE_TABLE[self.to_sq()],
            Axis::Rank => RANK_TABLE[self.to_sq()],
            Axis::Diagonal => DIAG_TABLE[self.to_sq()],
            Axis::AntiDiagonal => ADIAG_TABLE[self.to_sq()],
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
