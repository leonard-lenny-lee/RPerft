use super::*;
use position::Position;
use movelist::Move;

/// Create a new position by applying move data to a position
pub fn make_move(pos: &Position, mv: &Move) -> Position {
    
    // Create a copy of the current position to modify
    let mut new_pos = pos.clone();
    // Unpack move data
    let target = mv.target();
    let src = mv.src();
    let moved_piece = new_pos.our_piece_at(src);
    // Common operations for all moves
    update_shared_bitboards(&mut new_pos, target, src);
    update_our_bitboards(&mut new_pos, target, src, moved_piece);
    update_castling_rights(&mut new_pos, src, moved_piece);
    update_halfmove_clock(&mut new_pos, mv, moved_piece);
    update_fullmove_clock(&mut new_pos, );
    // Set en passant target sq to empty, this will be set to a value only
    // if the move was a pawn double push
    new_pos.data.en_passant_target_sq = EMPTY_BB;

    // Implement special updates
    let (flag_one, flag_two) = (mv.flag_one(), mv.flag_two());
    match flag_one {
        0 => match flag_two {
            0 => (), // Quiet Move
            64 => execute_double_push(&mut new_pos, target),
            _ => execute_castling(&mut new_pos, mv, target),
        },
        64 => match flag_two {
            0 => execute_capture(&mut new_pos, target),
            _ => execute_en_passant(&mut new_pos, target),
        },
        128 => execute_promotions(&mut new_pos, mv, target),
        _ => {
            debug_assert!(flag_one == 192); // Promo-capture
            execute_capture(&mut new_pos, target);
            execute_promotions(&mut new_pos, mv, target);
        }
    }
    // Change the turn and state
    new_pos.change_state();
    new_pos.key.update_key(
        moved_piece, src, target, &new_pos.data, &pos.data
    );
    return new_pos
}

#[inline]
fn update_shared_bitboards(pos: &mut Position, target: BB, src: BB) {
    // Source squares must be free after a move
    pos.data.free |= src; 
    pos.data.occ &= !src;
    // Target sqaures must be occupied after a move
    pos.data.free &= !target; 
    pos.data.occ |= target;
}

#[inline]
fn update_our_bitboards(
    pos: &mut Position, target: BB, src: BB, moved_piece: usize
) {
    let our_pieces = pos.mut_our_pieces();
    let move_mask = src | target;
    // Our bitboards must be flipped at target and source
    our_pieces[moved_piece] ^= move_mask; 
    our_pieces.any ^= move_mask;
}

#[inline]
fn execute_capture(pos: &mut Position, target: BB) {
    let captured_piece = pos.their_piece_at(target);
    let their_pieces = pos.mut_their_pieces();
    // If capture has taken place, then their bitboard must be unset at
    // target positions
    their_pieces[captured_piece] ^= target;
    their_pieces.any ^= target;
    // If their rook has been captured, check if it's a rook from on their
    // starting square. If so, unset their corresponding castling right
    pos.data.castling_rights &= !target;
    // Update Zobrist hash with the capture
    pos.key.update_square(
        captured_piece, target, !pos.data.white_to_move
    )
}

#[inline]
fn update_castling_rights(pos: &mut Position, src: BB, moved_piece: usize) {
    // If our king has moved, either normally or through castling,
    // remove all further rights to castle
    if moved_piece == Piece::King.value() {
        pos.data.castling_rights &= !pos.our_backrank()
    }
    // If our rook has moved from its starting square, remove rights for
    // that side
    pos.data.castling_rights &= !src
}

#[inline]
fn update_halfmove_clock(pos: &mut Position, mv: &Move, moved_piece: usize) {
    // Reset the half move clock for pawn move of any capture
    if mv.is_capture() || moved_piece == Piece::Pawn.value() {
        pos.data.halfmove_clock = 0
    } else {
        pos.data.halfmove_clock += 1
    }
}

#[inline]
fn update_fullmove_clock(pos: &mut Position) {
    // Increment the full move clock if black has moved
    pos.data.fullmove_clock += !pos.data.white_to_move as i8
}

#[inline]
fn execute_promotions(pos: &mut Position, mv: &Move, target: BB) {
    let our_pieces = pos.mut_our_pieces();
    let promotion_piece = mv.promotion_piece();
    // Set target square on promotion piece bitboard
    our_pieces[promotion_piece] ^= target;
    // Unset the pawn from our pawn bitboard
    our_pieces[Piece::Pawn.value()] ^= target;
    // Update the Zobrist hashes
    pos.key.update_square(
        Piece::Pawn.value(), target, pos.data.white_to_move
    );
    pos.key.update_square(
        promotion_piece, target, pos.data.white_to_move
    );
}

#[inline]
fn execute_castling(pos: &mut Position, mv: &Move, target: BB) {
    let our_pieces = pos.mut_our_pieces();
    // For castling moves, we also need the update our rook and shared
    // bitboards
    let (rook_src,rook_target) = if mv.is_short_castle() {
        // For kingside castle, the rook has transported from a
        // position one east of the target square to one west
        (target.east_one(), target.west_one())
    } else {
        // For the queenside castle, the rook has transported from
        // a position 2 squares west of the target square to the
        // position 1 east of the target sqaure
        debug_assert!(mv.is_long_castle());
        (target.west_two(), target.east_one())
    };
    let castle_mask = rook_src | rook_target;
    our_pieces[Piece::Rook.value()] ^= castle_mask;
    our_pieces[Piece::Any.value()] ^= castle_mask;
    // We also need to updated shared bitboards
    pos.data.occ ^= castle_mask;
    pos.data.free ^= castle_mask;
    // Update the Zobrist hash for the rook movement
    pos.key.update_moved_piece(
        Piece::Rook.value(), rook_src, rook_target,
        pos.data.white_to_move
    )
}

#[inline]
fn execute_en_passant(pos: &mut Position, target: BB) {
    // If white made the en passant capture, then the square at which the 
    // capture takes place is on square south of the target square and the
    // opposite for black
    let ep_capture_sq = pos.pawn_sgl_push_srcs(target);
    // Reflect the capture on the opponent bitboards
    let their_pieces = pos.mut_their_pieces();
    their_pieces[Piece::Pawn.value()] ^= ep_capture_sq;
    their_pieces[Piece::Any.value()] ^= ep_capture_sq;
    // We also need to update shared bitboards
    pos.data.occ ^= ep_capture_sq;
    pos.data.free ^= ep_capture_sq;
    // Update Zobrist hash
    pos.key.update_square(
        Piece::Pawn.value(), ep_capture_sq,
        !pos.data.white_to_move
    )
}

#[inline]
fn execute_double_push(pos: &mut Position, target: BB) {
    // If white made the double pawn push, then the ep target
    // square must be one square south of the target square and vice versa
    // for black
    let en_passant_target = pos.pawn_sgl_push_srcs(target);
    pos.data.en_passant_target_sq = en_passant_target
}