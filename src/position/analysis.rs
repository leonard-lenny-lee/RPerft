/// Module containing methods to extract information from a position
use super::*;
use types::{Axis, Piece};

impl Position {
    /// Return a bitboard with all squares the opponent pieces are attacking
    pub fn opponent_attack_squares(&self) -> BitBoard {
        // Remove our king from the occupancy bitboard to prevent the king from
        // blocking other squares attacked by sliding pieces
        let occupied = self.occupied ^ self.us.king;

        let mut unsafe_squares = constants::bb::EMPTY;
        unsafe_squares |=
            self.capture_left_rev(self.them.pawn) | self.capture_right_rev(self.them.pawn);
        unsafe_squares |= (self.them.rook | self.them.queen).ks_rook_attacks(occupied);
        unsafe_squares |= (self.them.bishop | self.them.queen).ks_bishop_attacks(occupied);
        unsafe_squares |= self.them.knight.generate_knight_attacks();
        unsafe_squares |= self.them.king.lookup_king_attacks();
        unsafe_squares
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn opponent_checkers(&self) -> BitBoard {
        let mut checkers = constants::bb::EMPTY;
        checkers |=
            (self.capture_left(self.us.king) | self.capture_right(self.us.king)) & self.them.pawn;
        checkers |=
            self.us.king.magic_rook_attacks(self.occupied) & (self.them.rook | self.them.queen);
        checkers |=
            self.us.king.magic_bishop_attacks(self.occupied) & (self.them.bishop | self.them.queen);
        checkers |= self.us.king.lookup_knight_attacks() & self.them.knight;
        checkers
    }

    /// Return a bitboard of all pinned pieces
    pub fn our_pinned_pieces(&self) -> BitBoard {
        let rooks = self.them.rook | self.them.queen;
        let bishops = self.them.bishop | self.them.queen;
        let occ = self.occupied;

        // Pinned pieces are located where a king's "attack ray" meets an attacking piece's attack ray,
        // cast along the same axis
        let mut pinned = constants::bb::EMPTY;
        pinned |= self.us.king.hyp_quint(occ, Axis::File) & rooks.ks_file_attacks(occ);
        pinned |= self.us.king.hyp_quint(occ, Axis::Rank) & rooks.ks_rank_attacks(occ);
        pinned |= self.us.king.hyp_quint(occ, Axis::Diagonal) & bishops.ks_diag_attacks(occ);
        pinned |= self.us.king.hyp_quint(occ, Axis::AntiDiagonal) & bishops.ks_adiag_attacks(occ);
        pinned
    }

    /// Return a bitboard with all the squares our pieces are attacking
    pub fn our_attack_squares(&self) -> BitBoard {
        let mut targets = constants::bb::EMPTY;
        targets |= self.capture_left(self.us.pawn) | self.capture_right(self.us.pawn);
        targets |= (self.us.rook | self.us.queen).ks_rook_attacks(self.occupied);
        targets |= (self.us.bishop | self.us.queen).ks_bishop_attacks(self.occupied);
        targets |= self.us.king.lookup_king_attacks();
        targets |= self.us.knight.generate_knight_attacks();
        targets
    }

    /// Check that in the position, we cannot capture their king. If so, it's an illegal position
    pub fn check_legal(&self) -> Result<(), ()> {
        if (self.our_attack_squares() & self.them.king).is_not_empty() {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl BBSet {
    /// Identify the piece type at a given bitboard
    pub fn piecetype_at(&self, bb: BitBoard) -> Option<Piece> {
        for pt in types::PIECES.iter() {
            if (self[*pt] & bb).is_not_empty() {
                return Some(*pt);
            }
        }
        return None;
    }
}
