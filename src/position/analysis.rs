/// Module containing methods to extract information from a position
use super::*;
use types::Piece;

impl Position {
    /// Return a bitboard with all squares the opponent pieces are attacking
    pub fn unsafe_sq(&self) -> BitBoard {
        // Remove our king from the occupancy bitboard to prevent the king from
        // blocking other squares attacked by sliding pieces
        let occ = self.occ ^ self.us.king;

        let mut unsafe_sq = constants::bb::EMPTY;
        unsafe_sq |= self.l_cap_back(self.them.pawn) | self.r_cap_back(self.them.pawn);
        unsafe_sq |= (self.them.rook | self.them.queen).ks_rook_attacks(occ);
        unsafe_sq |= (self.them.bishop | self.them.queen).ks_bishop_attacks(occ);
        unsafe_sq |= self.them.knight.generate_knight_attacks();
        unsafe_sq |= self.them.king.king_attacks_lu();
        unsafe_sq
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn checkers(&self) -> BitBoard {
        let mut checkers = constants::bb::EMPTY;
        checkers |= (self.l_cap(self.us.king) | self.r_cap(self.us.king)) & self.them.pawn;
        checkers |= self.us.king.rook_magic_lu(self.occ) & (self.them.rook | self.them.queen);
        checkers |= self.us.king.bishop_magic_lu(self.occ) & (self.them.bishop | self.them.queen);
        checkers |= self.us.king.knight_attacks_lu() & self.them.knight;
        checkers
    }

    /// Return a bitboard of all pinned pieces
    pub fn pinned(&self) -> BitBoard {
        let rooks = self.them.rook | self.them.queen;
        let bishops = self.them.bishop | self.them.queen;
        let occ = self.occ;

        // Pinned pieces are located where a king's "attack ray" meets an attacking piece's attack ray,
        // cast along the same axis
        let mut pinned = constants::bb::EMPTY;
        pinned |= self.us.king.hq_file_attacks(occ) & rooks.ks_file_attacks(occ);
        pinned |= self.us.king.hq_rank_attacks(occ) & rooks.ks_rank_attacks(occ);
        pinned |= self.us.king.hq_diag_attacks(occ) & bishops.ks_diag_attacks(occ);
        pinned |= self.us.king.hq_adiag_attacks(occ) & bishops.ks_adiag_attacks(occ);
        pinned
    }

    /// Return a bitboard with all the squares our pieces are attacking
    fn attack_sq(&self) -> BitBoard {
        let mut targets = constants::bb::EMPTY;
        targets |= self.l_cap(self.us.pawn) | self.r_cap(self.us.pawn);
        targets |= (self.us.rook | self.us.queen).ks_rook_attacks(self.occ);
        targets |= (self.us.bishop | self.us.queen).ks_bishop_attacks(self.occ);
        targets |= self.us.king.king_attacks_lu();
        targets |= self.us.knight.generate_knight_attacks();
        targets
    }

    /// Check that in the position, we cannot capture their king. If so, it's an illegal position
    pub fn check_legal(&self) -> Result<(), ()> {
        if (self.attack_sq() & self.them.king).is_not_empty() {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl BitBoardSet {
    /// Identify the piece type at a given bitboard
    pub fn pt_at(&self, bb: BitBoard) -> Option<Piece> {
        for pt in types::PIECES.iter() {
            if (self[*pt] & bb).is_not_empty() {
                return Some(*pt);
            }
        }
        return None;
    }
}
