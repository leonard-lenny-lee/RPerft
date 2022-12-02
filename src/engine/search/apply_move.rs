use super::*;
use position::Position;
use crate::disc;

/// Create a new position by applying move data to a position
pub fn apply_move(pos: &Position, mv: &Move) -> Position {

    // Create a copy of the current position to modify
    let mut new_pos = Position::new(pos.data.clone());
    // Unpack move data
    let target = mv.target();
    let src = mv.src();
    let moved_piece = pos.our_piece_at(src);
    // Common operations for all moves
    modify_universal_bitboards(&mut new_pos, target, src);
    execute_common_operations(&mut new_pos, target, src, mv, moved_piece);
    // Free the squares on the their bitboards if the piece is a capture
    if mv.is_capture() {
        execute_capture_operations(&mut new_pos, target, src, mv)
    }
    set_castling_rights(&mut new_pos, target, src, mv, moved_piece);
    set_halfmove_clock(&mut new_pos, mv, moved_piece);
    // Set en passant target sq to empty, this will be set to a value only
    // if the move was a pawn double push
    new_pos.data.en_passant_target_sq = EMPTY_BB;

    match mv.special_move_flag {
        SpecialMove::None => (),
        SpecialMove::Promotion => {
            execute_promotion_operations(&mut new_pos, mv, target)
        },
        SpecialMove::Castling => {
            execute_castling_operations(&mut new_pos, mv, target, moved_piece)
        },
        SpecialMove::EnPassant => {
            assert!(pos.data.en_passant_target_sq != EMPTY_BB);
            execute_en_passant_operations(&mut new_pos, mv, target)
        },
        SpecialMove::DoublePush => {
            execute_double_push_operations(&mut new_pos, mv, target)
        }
    }
    // Change the turn and state
    new_pos.change_state();
    return new_pos
}

fn modify_universal_bitboards(pos: &mut Position, target: u64, src: u64) {
    // Source squares must be free after a move
    pos.data.free |= src; 
    pos.data.occ &= !src;
    // Target sqaures must be occupied after a move
    pos.data.free &= !target; 
    pos.data.occ |= target;
}

fn execute_common_operations(pos: &mut Position, target: u64, src: u64, mv: &Move, moved_piece: usize) {
    let our_pieces = pos.mut_our_pieces();
    let move_mask = src | target;
    // Our bitboards must be flipped at target and source
    our_pieces.xor_assign(moved_piece, move_mask); 
    our_pieces.any ^= move_mask;
}

fn execute_capture_operations(pos: &mut Position, target: u64, src: u64, mv: &Move) {
    let captured_piece = pos.their_piece_at(target);
    let their_pieces = pos.mut_their_pieces();
    // If capture has taken place, then their bitboard must be unset at the
    // target positions
    their_pieces.xor_assign(captured_piece, target);
    their_pieces.any ^= target;
    // If their rook has been captured, check if it's a rook from on their
    // starting square. If so, unset their corresponding castling right
    if captured_piece == 2 {
        if target == pos.their_ks_rook_starting_sq() {
            pos.set_their_ksc(false)
        }
        if target == pos.their_qs_rook_starting_sq() {
            pos.set_their_qsc(false)
        }
    }
}

fn set_castling_rights(pos: &mut Position, target: u64, src: u64, mv: &Move, moved_piece: usize) {
    // If our king has moved, either normally or through castling, immediately
    // remove all further rights to castle
    if moved_piece == disc!(Piece::King) {
        pos.set_our_ksc(false);
        pos.set_our_qsc(false);
    }
    // If our rook has moved from its starting square, remove rights to castle
    // that side
    if moved_piece == disc!(Piece::Rook) {
        if src == pos.our_ks_rook_starting_sq() {
            pos.set_our_ksc(false)
        }
        if src == pos.our_qs_rook_starting_sq() {
            pos.set_our_qsc(false)
        }
    }
}

fn set_halfmove_clock(pos: &mut Position, mv: &Move, moved_piece: usize) {
    // Reset the half move clock if a pawn is moved or a capture has occurred
    if mv.is_capture() || moved_piece == disc!(Piece::Pawn) {
        pos.data.halfmove_clock = 0
    } else {
        pos.data.halfmove_clock += 1
    }
}

fn execute_promotion_operations(pos: &mut Position, mv: &Move, target: u64) {
    let our_pieces = pos.mut_our_pieces();
    // Set target square on promotion piece bitboard
    our_pieces.bit_or_assign(disc!(mv.promotion_piece), target);
    // Unset the pawn from our pawn bitboard
    our_pieces.xor_assign(disc!(Piece::Pawn), target)
}

fn execute_castling_operations(pos: &mut Position, mv: &Move, target: u64, moved_piece: usize) {
    let our_pieces = pos.mut_our_pieces();
    assert!(moved_piece == disc!(Piece::King));
    // For castling moves, we also need the update our rook and any bitboards
    // Calculate if kingside or queenside castle
    let castle_mask: u64;
    if target.trailing_zeros() % 8 == 6 {
        // For kingside castle, the rook has transported from a
        // position one east of the target square to one west
        castle_mask = bt::east_one(target) | bt::west_one(target);
    } else {
        // For the queenside castle, the rook has transported from
        // a position 2 squares west of the target square to the
        // position 1 east of the target sqaure
        assert!(target.trailing_zeros() % 8 == 2);
        castle_mask = bt::east_one(target) | bt::west_two(target);
    }
    our_pieces.xor_assign(disc!(Piece::Rook), castle_mask);
    our_pieces.xor_assign(disc!(Piece::Any), castle_mask);
    // We also need to modify the universal occupancy bitboards
    pos.data.occ ^= castle_mask;
    pos.data.free ^= castle_mask;
}

fn execute_en_passant_operations(pos: &mut Position, mv: &Move, target: u64) {
    // If white made the en passant capture, then the square at which the 
    // capture takes place is on square south of the target square and the
    // opposite for black
    let ep_capture_sq = pos.pawn_sgl_push_srcs(target);
    // Reflect the capture on the opponent bitboards
    let their_pieces = pos.mut_their_pieces();
    their_pieces.xor_assign(disc!(Piece::Pawn), ep_capture_sq);
    their_pieces.xor_assign(disc!(Piece::Any), ep_capture_sq);
    // We also need to modify the universal occupancy bitboards
    pos.data.occ ^= ep_capture_sq;
    pos.data.free ^= ep_capture_sq;
}

fn execute_double_push_operations(pos: &mut Position, mv: &Move, target: u64) {
    // If white made the double pawn push, then the ep target
    // square must be one square south of the target square and vice versa
    // for black
    let en_passant_target = pos.pawn_sgl_push_srcs(target);
    pos.data.en_passant_target_sq = en_passant_target
}

#[cfg(test)]
mod test_apply_move {
}