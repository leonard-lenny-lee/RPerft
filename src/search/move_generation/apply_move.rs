use super::*;

/// Create a new position by applying move data to a position
pub fn apply_move(pos: &Position, mv: &Move) -> Position {

    // Create a copy of the current position to modify
    let mut new_pos = Position::new(pos.data.clone());
    // Common operations for all moves
    modify_universal_bitboards(&mut new_pos, mv);
    execute_common_operations(&mut new_pos, mv);
    // Free the squares on the their bitboards if the piece is a capture
    if mv.is_capture {
        execute_capture_operations(&mut new_pos, mv)
    }
    set_castling_rights(&mut new_pos, mv);
    set_halfmove_clock(&mut new_pos, mv);
    // Set en passant target sq to empty, this will be set to a value only
    // if the move was a pawn double push
    new_pos.data.en_passant_target_sq = EMPTY_BB;

    match mv.special_move_flag {
        SpecialMove::None => (),
        SpecialMove::Promotion => {
            execute_promotion_operations(&mut new_pos, mv)
        },
        SpecialMove::Castling => {
            execute_castling_operations(&mut new_pos, mv)
        },
        SpecialMove::EnPassant => {
            assert!(pos.data.en_passant_target_sq != EMPTY_BB);
            execute_en_passant_operations(&mut new_pos, mv)
        },
        SpecialMove::DoublePush => {
            execute_double_push_operations(&mut new_pos, mv)
        }
    }
    // Change the turn and state
    new_pos.change_state();
    return new_pos
}

fn modify_universal_bitboards(pos: &mut Position, mv: &Move) {
    // Source squares must be free after a move
    pos.data.free |= mv.src; 
    pos.data.occ &= !mv.src;
    // Target sqaures must be occupied after a move
    pos.data.free &= !mv.target; 
    pos.data.occ |= mv.target;
}

fn execute_common_operations(pos: &mut Position, mv: &Move) {
    let our_pieces = pos.mut_our_pieces();
    let move_mask = mv.src | mv.target;
    // Our bitboards must be flipped at target and source
    our_pieces.xor_assign(disc!(mv.moved_piece), move_mask); 
    our_pieces.any ^= move_mask;
}

fn execute_capture_operations(pos: &mut Position, mv: &Move) {
    let their_pieces;
    if pos.data.white_to_move {
        their_pieces = &mut pos.data.w_pieces;
    } else {
        their_pieces = &mut pos.data.b_pieces;
    }
    // If capture has taken place, then their bitboard must be unset at the
    // target positions
    their_pieces.xor_assign(disc!(mv.captured_piece), mv.target);
    their_pieces.any ^= mv.target;
}

fn set_castling_rights(pos: &mut Position, mv: &Move) {
    // If their rook has been captured, check if it's a rook from on their
    // starting square. If so, unset their corresponding castling right
    if matches!(mv.captured_piece, Piece::Rook) {
        if mv.target == pos.their_ks_rook_starting_sq() {
            pos.set_their_ksc(false)
        }
        if mv.target == pos.their_qs_rook_starting_sq() {
            pos.set_their_qsc(false)
        }
    }
    // If our king has moved, either normally or through castling, immediately
    // remove all further rights to castle
    if matches!(mv.moved_piece, Piece::King) {
        pos.set_our_ksc(false);
        pos.set_our_qsc(false);
    }
    // If our rook has moved from its starting square, remove rights to castle
    // that side
    if matches!(mv.moved_piece, Piece::Rook) {
        if mv.src == pos.our_ks_rook_starting_sq() {
            pos.set_our_ksc(false)
        }
        if mv.src == pos.our_qs_rook_starting_sq() {
            pos.set_our_qsc(false)
        }
    }
}

fn set_halfmove_clock(pos: &mut Position, mv: &Move) {
    // Reset the half move clock if a pawn is moved or a capture has occurred
    if mv.is_capture || matches!(mv.moved_piece, Piece::Pawn) {
        pos.data.halfmove_clock = 0
    } else {
        pos.data.halfmove_clock += 1
    }
}

fn execute_promotion_operations(pos: &mut Position, mv: &Move) {
    let our_pieces = pos.mut_our_pieces();
    // Set target square on promotion piece bitboard
    our_pieces.bit_or_assign(disc!(mv.promotion_piece), mv.target);
    // Unset the pawn from our pawn bitboard
    our_pieces.xor_assign(disc!(Piece::Pawn), mv.target)
}

fn execute_castling_operations(pos: &mut Position, mv: &Move) {
    let our_pieces = pos.mut_our_pieces();
    assert!(matches!(mv.moved_piece, Piece::King));
    // For castling moves, we also need the update our rook and any bitboards
    // Calculate if kingside or queenside castle
    let castle_mask: u64;
    if mv.target.trailing_zeros() % 8 == 6 {
        // For kingside castle, the rook has transported from a
        // position one east of the target square to one west
        castle_mask = bt::east_one(mv.target) | bt::west_one(mv.target);
    } else {
        // For the queenside castle, the rook has transported from
        // a position 2 squares west of the target square to the
        // position 1 east of the target sqaure
        assert!(mv.target.trailing_zeros() % 8 == 2);
        castle_mask = bt::east_one(mv.target) | bt::west_two(mv.target);
    }
    our_pieces.xor_assign(disc!(Piece::Rook), castle_mask);
    our_pieces.xor_assign(disc!(Piece::Any), castle_mask);
}

fn execute_en_passant_operations(pos: &mut Position, mv: &Move) {
    // If white made the en passant capture, then the square at which the 
    // capture takes place is on square south of the target square and the
    // opposite for black
    let ep_capture_sq = pos.pawn_sgl_push_srcs(mv.target);
    // Reflect the capture on the opponent bitboards
    let their_pieces = pos.mut_their_pieces();
    their_pieces.xor_assign(disc!(Piece::Pawn), ep_capture_sq);
    their_pieces.xor_assign(disc!(Piece::Any), ep_capture_sq);
}

fn execute_double_push_operations(pos: &mut Position, mv: &Move) {
    // If white made the double pawn push, then the ep target
    // square must be one square south of the target square and vice versa
    // for black
    let en_passant_target = pos.pawn_sgl_push_srcs(mv.target);
    pos.data.en_passant_target_sq = en_passant_target
}

#[cfg(test)]
mod test_apply_move {
}