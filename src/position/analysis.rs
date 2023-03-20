/// Module containing methods to extract information from a position
use super::*;
use types::{Axis, PieceType};

impl Position {
    /// Return a bitboard with all squares the opponent pieces are attacking
    pub fn unsafe_squares(&self) -> BB {
        let mut unsafe_squares = EMPTY_BB;
        // Remove our king from the occupancy bitboard for sliding piece move
        // generation to prevent the king from blocking other unsafe squares
        let occ = self.occupied ^ self.us().king;
        let their_pieces = self.them();
        // Calculate pawn attacks
        unsafe_squares |= self.unsafe_squares_pawn();
        // Calculate attacks in horizontal and vertical directions
        unsafe_squares |= (their_pieces.rook | their_pieces.queen).rook_attacks(occ);
        // Calculate attacks in the diagonal and anti-diagonal directions
        unsafe_squares |= (their_pieces.bishop | their_pieces.queen).bishop_attacks(occ);
        // Calculate knight attacks
        for sq in their_pieces.knight {
            unsafe_squares |= sq.lu_knight_attacks()
        }
        // Calculate king attacks
        unsafe_squares |= their_pieces.king.lu_king_attacks();
        return unsafe_squares;
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn find_checkers(&self) -> BB {
        let mut checkers = EMPTY_BB;
        let king = self.us().king;
        let their_pieces = self.them();
        // Find checking pawns
        checkers |= self.their_checking_pawns();
        // Find checkers along the files and ranks
        checkers |= king.lu_rook_attacks(self.occupied) & (their_pieces.rook | their_pieces.queen);
        // Find checkers along the diagonals
        checkers |=
            king.lu_bishop_attacks(self.occupied) & (their_pieces.bishop | their_pieces.queen);
        // Find knight checkers
        checkers |= king.lu_knight_attacks() & their_pieces.knight;
        checkers
    }

    /// Return a bitboard of all pinned pieces
    pub fn pinned_pieces(&self) -> BB {
        let (king, their_pieces) = (self.us().king, self.them());
        let file_rank_pieces = their_pieces.rook | their_pieces.queen;
        let diag_adiag_pieces = their_pieces.bishop | their_pieces.queen;
        let occ = self.occupied;

        // Pinned pieces are located where a king's "attack ray" meets an
        // attacking piece's attack ray, cast along the same axis
        let file_pins = king.hyp_quint(occ, Axis::File) & file_rank_pieces.file_attacks(occ);
        let rank_pins = king.hyp_quint(occ, Axis::Rank) & file_rank_pieces.rank_attacks(occ);
        let diag_pins = king.hyp_quint(occ, Axis::Diagonal) & diag_adiag_pieces.diag_attacks(occ);
        let adiag_pins =
            king.hyp_quint(occ, Axis::AntiDiagonal) & diag_adiag_pieces.adiag_attacks(occ);

        file_pins | rank_pins | diag_pins | adiag_pins
    }

    /// Return a bitboard with all the squares our pieces are attacking
    pub fn target_squares(&self) -> BB {
        let mut target_squares = EMPTY_BB;
        let our_pieces = self.us();
        // Pawn attacks
        target_squares |= self.left_capture(our_pieces.pawn) | self.right_capture(our_pieces.pawn);
        // Horizontal and vertical attacks
        target_squares |= (our_pieces.rook | our_pieces.queen).rook_attacks(self.occupied);
        // Diagonal and antidiagonal attacks
        target_squares |= (our_pieces.bishop | our_pieces.queen).bishop_attacks(self.occupied);
        // Knight attacks
        for sq in our_pieces.knight {
            target_squares |= sq.lu_knight_attacks()
        }
        // King attacks
        target_squares |= our_pieces.king.lu_king_attacks();
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
            bb.to_index()
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
            bb.to_index()
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
    pub fn check_legality(&self) -> Result<(), RuntimeError> {
        if (self.target_squares() & self.them().king).is_not_empty() {
            Err(RuntimeError::ParseFenError)
        } else {
            Ok(())
        }
    }
}

impl BBSet {
    /// Identify the piece type at a given square
    pub fn pt_at(&self, bb: BB) -> PieceType {
        for pt in PieceType::iterpieces() {
            if (self[*pt] & bb).is_not_empty() {
                return *pt;
            }
        }
        panic!("no piece at sq {}", bb.to_index());
    }
}
