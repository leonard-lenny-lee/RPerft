/// Compile time generated lookup tables.
use super::*;
use types::Axis;

const KNIGHT_ATTACKS: [BB; 64] = {
    let mut maps: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        maps[i] = BB(1 << i).knight_attacks();
        i += 1;
    }
    maps
};

const KING_ATTACKS: [BB; 64] = {
    let mut maps: [BB; 64] = [BB(0); 64];
    let mut i = 0;
    while i < 64 {
        maps[i] = BB(1 << i).king_attacks();
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
