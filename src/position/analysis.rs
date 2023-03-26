/// Module containing methods to extract information from a position
use super::*;
use types::{Axis, PieceType};

impl Position {
    /// Return a bitboard with all squares the opponent pieces are attacking
    pub fn unsafe_sq(&self) -> BB {
        // Remove our king from the occupancy bitboard to prevent the king from
        // blocking other squares attacked by sliding pieces
        let occ = self.occ ^ self.us().king;
        let them = self.them();

        let mut unsafe_sqs = EMPTY_BB;

        unsafe_sqs |= self.pawn_attacks();
        unsafe_sqs |= (them.rook | them.queen).rook_attacks(occ);
        unsafe_sqs |= (them.bishop | them.queen).bishop_attacks(occ);
        unsafe_sqs |= them.knight.knight_attacks();
        unsafe_sqs |= them.king.king_lu();

        return unsafe_sqs;
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn checkers(&self) -> BB {
        let king = self.us().king;
        let them = self.them();

        let mut checkers = EMPTY_BB;

        checkers |= self.pawn_checkers();
        checkers |= king.rook_lu(self.occ) & (them.rook | them.queen);
        checkers |= king.bishop_lu(self.occ) & (them.bishop | them.queen);
        checkers |= king.knight_lu() & them.knight;

        return checkers;
    }

    /// Return a bitboard of all pinned pieces
    pub fn pinned(&self) -> BB {
        let king = self.us().king;
        let them = self.them();
        let rooks = them.rook | them.queen;
        let bishops = them.bishop | them.queen;
        let occ = self.occ;

        // Pinned pieces are located where a king's "attack ray" meets an
        // attacking piece's attack ray, cast along the same axis
        let mut pinned = EMPTY_BB;
        pinned |= king.hyp_quint(occ, Axis::File) & rooks.file_attacks(occ);
        pinned |= king.hyp_quint(occ, Axis::Rank) & rooks.rank_attacks(occ);
        pinned |= king.hyp_quint(occ, Axis::Diagonal) & bishops.diag_attacks(occ);
        pinned |= king.hyp_quint(occ, Axis::AntiDiagonal) & bishops.adiag_attacks(occ);

        return pinned;
    }

    /// Return a bitboard with all the squares our pieces are attacking
    pub fn target_squares(&self) -> BB {
        let mut target_squares = EMPTY_BB;
        let our_pieces = self.us();
        // Pawn attacks
        target_squares |= self.lcap(our_pieces.pawn) | self.rcap(our_pieces.pawn);
        // Horizontal and vertical attacks
        target_squares |= (our_pieces.rook | our_pieces.queen).rook_attacks(self.occ);
        // Diagonal and antidiagonal attacks
        target_squares |= (our_pieces.bishop | our_pieces.queen).bishop_attacks(self.occ);
        // Knight attacks
        for sq in our_pieces.knight {
            target_squares |= sq.knight_lu()
        }
        // King attacks
        target_squares |= our_pieces.king.king_lu();
        return target_squares;
    }

    /// Identify which opponent piece is a particular position as the index
    /// of the array representation of the pieceset
    pub fn their_piece_at(&self, bb: BB) -> PieceType {
        debug_assert!(bb.pop_count() == 1);
        let them = self.them();
        for pt in PieceType::iterpieces() {
            if (them[*pt] & bb).is_not_empty() {
                return *pt;
            }
        }
        panic!(
            "their_piece_at could not locate the requested bit {}",
            bb.to_sq()
        );
    }

    /// Identify which of our pieces is a particular position as the index
    /// of the array representation of the pieceset
    pub fn our_piece_at(&self, bb: BB) -> PieceType {
        debug_assert!(bb.pop_count() == 1);
        let us = self.us();
        for pt in PieceType::iterpieces() {
            if (us[*pt] & bb).is_not_empty() {
                return *pt;
            }
        }
        panic!(
            "their_piece_at could not locate the requested bit {}",
            bb.to_sq()
        );
    }

    /// Identify if the piece at the specified square is a sliding piece
    pub fn their_piece_at_is_slider(&self, n: BB) -> bool {
        matches!(
            self.their_piece_at(n),
            PieceType::Rook | PieceType::Bishop | PieceType::Queen
        )
    }

    /// Check that in the position, we cannot capture their king. If so, it's
    /// an illegal position
    pub fn check_legal(&self) -> Result<(), RuntimeError> {
        if (self.target_squares() & self.them().king).is_not_empty() {
            Err(RuntimeError::ParseFenError)
        } else {
            Ok(())
        }
    }
}

impl BBSet {
    /// Identify the piece type at a given square
    pub fn pt_at(&self, bb: BB) -> Option<PieceType> {
        for pt in PieceType::iterpieces() {
            if (self[*pt] & bb).is_not_empty() {
                return Some(*pt);
            }
        }
        return None;
    }
}
