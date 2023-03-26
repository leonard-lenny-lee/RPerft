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

        unsafe_sqs |= self.lcap_back(them.pawn) | self.rcap_back(them.pawn);
        unsafe_sqs |= (them.rook | them.queen).rook_attacks(occ);
        unsafe_sqs |= (them.bishop | them.queen).bishop_attacks(occ);
        unsafe_sqs |= them.knight.knight_attacks();
        unsafe_sqs |= them.king.king_lu();

        return unsafe_sqs;
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn checkers(&self) -> BB {
        let (us, them) = self.us_them();

        let mut checkers = EMPTY_BB;

        checkers |= (self.lcap(us.king) | self.rcap(us.king)) & them.pawn;
        checkers |= us.king.rook_lu(self.occ) & (them.rook | them.queen);
        checkers |= us.king.bishop_lu(self.occ) & (them.bishop | them.queen);
        checkers |= us.king.knight_lu() & them.knight;

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
    pub fn attack_sq(&self) -> BB {
        let mut targets = EMPTY_BB;
        let us = self.us();

        targets |= self.lcap(us.pawn) | self.rcap(us.pawn);
        targets |= (us.rook | us.queen).rook_attacks(self.occ);
        targets |= (us.bishop | us.queen).bishop_attacks(self.occ);
        targets |= us.king.king_lu();
        targets |= us.knight.knight_attacks();

        return targets;
    }

    /// Check that in the position, we cannot capture their king. If so, it's
    /// an illegal position
    pub fn check_legal(&self) -> Result<(), RuntimeError> {
        if (self.attack_sq() & self.them().king).is_not_empty() {
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
