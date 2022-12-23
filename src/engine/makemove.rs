use super::*;
use position::Position;
use movelist::Move;

impl Position {

    /// Create a new position by applying move data to a position
    pub fn make_move(&self, mv: &Move) -> Position {
        
        // Create a copy of the current position to modify
        let mut new_pos = self.clone();
        // Unpack move data
        let target = mv.target();
        let src = mv.src();
        let moved_piece = new_pos.our_piece_at(src);
        // Common operations for all moves
        new_pos.modify_universal_bitboards(target, src);
        new_pos.execute_common_operations(target, src, moved_piece);
        // Free the squares on the their bitboards if the piece is a capture
        if mv.is_capture() {
            // En passant moves are marked as captures but must be treated 
            // differently
            if !mv.is_en_passant() {
                new_pos.execute_capture_operations(target)
            } else {
                new_pos.execute_en_passant_operations(target)
            }
        }
        new_pos.set_castling_rights(src, moved_piece);
        new_pos.set_halfmove_clock(mv, moved_piece);
        // Set en passant target sq to empty, this will be set to a value only
        // if the move was a pawn double push
        new_pos.data.en_passant_target_sq = EMPTY_BB;

        if mv.is_quiet() {
            // None
        } else if mv.is_promotion() {
            new_pos.execute_promotion_operations(mv, target)
        } else if mv.is_castle() {
            new_pos.execute_castling_operations(mv, target, moved_piece)
        } else if mv.is_double_pawn_push() {
            new_pos.execute_double_push_operations(target)
        }
        // Change the turn and state
        new_pos.change_state();
        new_pos.key.update_key(moved_piece, src, target, &self.data, &new_pos.data);
        return new_pos
    }

    fn modify_universal_bitboards(&mut self, target: BB, src: BB) {
        // Source squares must be free after a move
        self.data.free |= src; 
        self.data.occ &= !src;
        // Target sqaures must be occupied after a move
        self.data.free &= !target; 
        self.data.occ |= target;
    }

    fn execute_common_operations(
        &mut self, target: BB, src: BB, moved_piece: usize
    ) {
        let our_pieces = self.mut_our_pieces();
        let move_mask = src | target;
        // Our bitboards must be flipped at target and source
        our_pieces.bitxor_assign(moved_piece, move_mask); 
        our_pieces.any ^= move_mask;
    }

    fn execute_capture_operations(&mut self, target: BB) {
        let captured_piece = self.their_piece_at(target);
        let their_pieces = self.mut_their_pieces();
        // If capture has taken place, then their bitboard must be unset at the
        // target positions
        their_pieces.bitxor_assign(captured_piece, target);
        their_pieces.any ^= target;
        // If their rook has been captured, check if it's a rook from on their
        // starting square. If so, unset their corresponding castling right
        self.data.castling_rights &= !target;
        // Update Zobrist hash with the capture
        self.key.update_square(
            captured_piece, target, !self.data.white_to_move
        )
    }

    fn set_castling_rights(&mut self, src: BB, moved_piece: usize) {
        // If our king has moved, either normally or through castling, immediately
        // remove all further rights to castle
        if moved_piece == Piece::King.value() {
            self.data.castling_rights &= !self.our_backrank()
        }
        // If our rook has moved from its starting square, remove rights to castle
        // that side
        self.data.castling_rights &= !src
    }

    fn set_halfmove_clock(&mut self, mv: &Move, moved_piece: usize) {
        // Reset the half move clock if a pawn is moved or a capture has occurred
        if mv.is_capture() || moved_piece == Piece::Pawn.value() {
            self.data.halfmove_clock = 0
        } else {
            self.data.halfmove_clock += 1
        }
    }

    fn execute_promotion_operations(&mut self, mv: &Move, target: BB) {
        let our_pieces = self.mut_our_pieces();
        let promotion_piece = mv.promotion_piece();
        // Set target square on promotion piece bitboard
        our_pieces.bitor_assign(promotion_piece, target);
        // Unset the pawn from our pawn bitboard
        our_pieces.bitxor_assign(Piece::Pawn.value(), target);
        // Update the Zobrist hashes
        self.key.update_square(Piece::Pawn.value(), target, self.data.white_to_move);
        self.key.update_square(promotion_piece, target, self.data.white_to_move);
    }

    fn execute_castling_operations(
        &mut self, mv: &Move, target: BB, moved_piece: usize
    ) {
        let our_pieces = self.mut_our_pieces();
        debug_assert!(moved_piece == Piece::King.value());
        // For castling moves, we also need the update our rook and any bitboards
        // Calculate if kingside or queenside castle
        let rook_src: BB;
        let rook_target: BB;
        if mv.is_short_castle() {
            // For kingside castle, the rook has transported from a
            // position one east of the target square to one west
            rook_src = target.east_one();
            rook_target = target.west_one();
        } else {
            // For the queenside castle, the rook has transported from
            // a position 2 squares west of the target square to the
            // position 1 east of the target sqaure
            debug_assert!(target.to_index() % 8 == 2);
            rook_src = target.west_two();
            rook_target = target.east_one();
        }
        let castle_mask = rook_src | rook_target;
        our_pieces.bitxor_assign(Piece::Rook.value(), castle_mask);
        our_pieces.bitxor_assign(Piece::Any.value(), castle_mask);
        // We also need to modify the universal occupancy bitboards
        self.data.occ ^= castle_mask;
        self.data.free ^= castle_mask;
        // Update the Zobrist hash for the rook movement
        self.key.update_moved_piece(
            Piece::Rook.value(), rook_src, rook_target, self.data.white_to_move
        )
    }

    fn execute_en_passant_operations(&mut self, target: BB) {
        // If white made the en passant capture, then the square at which the 
        // capture takes place is on square south of the target square and the
        // opposite for black
        let ep_capture_sq = self.pawn_sgl_push_srcs(target);
        // Reflect the capture on the opponent bitboards
        let their_pieces = self.mut_their_pieces();
        their_pieces.bitxor_assign(Piece::Pawn.value(), ep_capture_sq);
        their_pieces.bitxor_assign(Piece::Any.value(), ep_capture_sq);
        // We also need to modify the universal occupancy bitboards
        self.data.occ ^= ep_capture_sq;
        self.data.free ^= ep_capture_sq;
        // Update Zobrist hash
        self.key.update_square(
            Piece::Pawn.value(), ep_capture_sq,
            !self.data.white_to_move
        )
    }

    fn execute_double_push_operations(&mut self, target: BB) {
        // If white made the double pawn push, then the ep target
        // square must be one square south of the target square and vice versa
        // for black
        let en_passant_target = self.pawn_sgl_push_srcs(target);
        self.data.en_passant_target_sq = en_passant_target
    }

}