use strum::IntoEnumIterator;
use super::*;

use position::{Position, analysis_tools};

/// The master move finding function - finds all legal moves in a
/// position and returns the list of legal moves as a vector of moves
pub fn find_moves(pos: &Position) -> Vec<Move> {
    // Initialise variables
    let mut move_vec: Vec<Move> = Vec::new();
    let unsafe_squares = analysis_tools::find_unsafe_squares(pos);
    let checkers = analysis_tools::find_checkers(pos);
    let pinned_pieces = analysis_tools::get_pinned_pieces_for(pos);
    // Number of pieces placing the king in check
    let n_checkers = checkers.count_ones();
    let mut capture_mask: u64 = FILLED_BB;
    let mut push_mask: u64 = FILLED_BB;
    // If the king is in double check, only king moves to safe squares are valid
    if n_checkers > 1 {
        find_king_moves(&mut move_vec, pos, unsafe_squares);
        return move_vec;
    }
    if n_checkers == 1 {
        // This means the king is in single check so moves are only legal if
        // 1. It moves the king out of check
        // 2. The attacking piece is captured
        // 3. The attacking piece is blocked, if the piece is a sliding piece
        capture_mask = checkers;
        if pos.their_piece_at_is_slider(checkers) {
            // If the attacker is a sliding piece, then check can be blocked by
            // another piece moving to the intervening squares
            push_mask = bt::connect_squares(
                checkers, pos.our_pieces().king
            )
        } else {
            // Not a slider so it can only be captured;
            // give no options to block
            push_mask = EMPTY_BB
        }
    }

    // Add all moves to the move vector
    find_single_pushes(&mut move_vec, pos, push_mask, pinned_pieces);
    find_double_pushes(&mut move_vec, pos, push_mask, pinned_pieces);
    find_right_captures(&mut move_vec, pos, capture_mask, pinned_pieces);
    find_left_captures(&mut move_vec, pos, capture_mask, pinned_pieces);
    find_knight_moves(
        &mut move_vec, pos, capture_mask,
        push_mask, pinned_pieces
    );
    find_king_moves(
        &mut move_vec, pos, unsafe_squares
    );
    for piece in SlidingPiece::iter() {
        find_sliding_moves(
            &mut move_vec, pos, piece, capture_mask, 
            push_mask, pinned_pieces
        )
    }
    // Castling is only allowed if not in check
    if n_checkers == 0 {
        find_castling_moves(&mut move_vec, pos, unsafe_squares);
    }
    find_en_passant_moves(
        &mut move_vec, pos, capture_mask, push_mask, pinned_pieces
    );
    return move_vec;
}

/// Move generation functions. These accept a mutable move vector reference as
/// an argument and pushes legal moves in a position to the move vector

/// Move generation function to find all pawn single pushes in a position.
pub fn find_single_pushes(
    move_vec: &mut Vec<Move>, pos: &Position, push_mask: u64,
    pinned_pieces: u64
) {
    let targets = pos.pawn_sgl_push_targets() & push_mask;
    let srcs = pos.pawn_sgl_push_srcs(targets);
    // Separate promoting pawns from non-promoting pawns
    let mut promotion_pawns = srcs & pos.promotion_rank();
    let mut normal_pawns = srcs ^ promotion_pawns;
    // Separate pinned pawns
    let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
    promotion_pawns ^= pinned_promotion_pawns;
    let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
    normal_pawns ^= pinned_normal_pawns;
    
    // Normal pawns
    while normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut normal_pawns);
        let target = pos.pawn_sgl_push(src);
        move_vec.push(Move::new_quiet_move(target, src))
    }
    // Promotion pawns
    while promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut promotion_pawns);
        let target = pos.pawn_sgl_push(src);
        push_promotions(move_vec, target, src)
    }
    // For pinned pieces, only allow moves towards / away from the king
    while pinned_normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_normal_pawns);
        let mut target = pos.pawn_sgl_push(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            move_vec.push(Move::new_quiet_move(target, src))
        }
    }
    while pinned_promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_promotion_pawns);
        let mut target = pos.pawn_sgl_push(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            push_promotions(move_vec, target, src)
        }
    }

}

/// Move generation function to find all pawn double pushes in a position
pub fn find_double_pushes(
    move_vec: &mut Vec<Move>, pos: &Position, push_mask: u64,
    pinned_pieces: u64
) {
    let targets = pos.pawn_dbl_push_targets() & push_mask;
    let mut srcs = pos.pawn_dbl_push_srcs(targets);
    let mut pinned_srcs = srcs & pinned_pieces;
    srcs ^= pinned_srcs;
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        let target = pos.pawn_dbl_push(src);    
        move_vec.push(Move::new_double_pawn_push(target, src))
    }
    // For pinned pieces, only allow moves towards / away from the king
    while pinned_srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_srcs);
        let mut target = pos.pawn_dbl_push(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            move_vec.push(Move::new_double_pawn_push(target, src))
        }
    }
}

/// Move generation function to find all pawn left captures in a position
pub fn find_left_captures(
    move_vec: &mut Vec<Move>, pos: &Position, capture_mask: u64,
    pinned_pieces: u64
) {
    let targets = pos.pawn_lcap_targets() & capture_mask;
    let srcs = pos.pawn_lcap_srcs(targets);
    // Separate promotion pawns from non-promoting pawns
    let mut promotion_pawns = srcs & pos.promotion_rank();
    let mut normal_pawns = srcs ^ promotion_pawns;
    // Separate pinned pawns
    let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
    promotion_pawns ^= pinned_promotion_pawns;
    let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
    normal_pawns ^= pinned_normal_pawns;

    // Normal pawns
    while normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut normal_pawns);
        let target = pos.pawn_left_capture(src);
        move_vec.push(Move::new_capture(target, src))
    }
    // Promotion pawns
    while promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut promotion_pawns);
        let target = pos.pawn_left_capture(src);
        push_promo_captures(move_vec, target, src)
    }
    // For pinned pieces, only allow moves towards / away from the king
    while pinned_normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_normal_pawns);
        let mut target = pos.pawn_left_capture(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            move_vec.push(Move::new_capture(target, src))
        }
    }
    while pinned_promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_promotion_pawns);
        let mut target = pos.pawn_left_capture(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            push_promo_captures(move_vec, target, src)
        }
    }
}

/// Move generation function to find all pawn right captures in a position
pub fn find_right_captures(
    move_vec: &mut Vec<Move>, pos: &Position, capture_mask: u64,
    pinned_pieces: u64
) {
    let targets = pos.pawn_rcap_targets() & capture_mask;
    let srcs = pos.pawn_rcap_srcs(targets);
    // Separate promotion pawns from non-promoting pawns
    let mut promotion_pawns = srcs & pos.promotion_rank();
    let mut normal_pawns = srcs ^ promotion_pawns;
    // Separate pinned pawns
    let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
    promotion_pawns ^= pinned_promotion_pawns;
    let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
    normal_pawns ^= pinned_normal_pawns;

    // Normal pawns
    while normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut normal_pawns);
        let target = pos.pawn_right_capture(src);
        move_vec.push(Move::new_capture(target, src))
    }
    // Promotion pawns
    while promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut promotion_pawns);
        let target = pos.pawn_right_capture(src);
        push_promo_captures(move_vec, target, src)
    }
    // For pinned pieces, only allow moves towards / away from the king
    while pinned_normal_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_normal_pawns);
        let mut target = pos.pawn_right_capture(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            move_vec.push(Move::new_capture(target, src))
        }
    }
    while pinned_promotion_pawns != EMPTY_BB {
        let src = bt::pop_lsb(&mut pinned_promotion_pawns);
        let mut target = pos.pawn_right_capture(src);
        target &= bt::ray_axis(pos.our_pieces().king, src);
        if target != EMPTY_BB {
            push_promo_captures(move_vec, target, src)
        }
    }
}


/// Move generation function for knights
pub fn find_knight_moves(
    move_vec: &mut Vec<Move>, pos: &Position, 
    capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    let mut srcs = our_pieces.knight;
    // Filter out knights which are pinned - pinned knights have no legal moves
    srcs &= !pinned_pieces;
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        let mut targets = MAPS.get_knight_map(src) & !our_pieces.any;
        // Only allow moves which either capture a checking piece or blocks
        // the check. These masks should be a FILLED_BB when no check.
        targets &= capture_mask | push_mask;
        find_quiet_moves_and_captures(move_vec, pos, targets, src)
    }
}

/// Move generation function for kings
pub fn find_king_moves(
    move_vec: &mut Vec<Move>, pos: &Position,
    unsafe_squares: u64
) {
    let our_pieces = pos.our_pieces();
    let src = our_pieces.king;
    let mut targets = MAPS.get_king_map(src) & !our_pieces.any;
    // Remove unsafe squares i.e. squares attacked by opponent pieces
    // from the available target sqaures for the king
    targets &= !unsafe_squares;
    find_quiet_moves_and_captures(move_vec, pos, targets, src)
}

/// General move generation function for sliding pieces - Rooks, Bishops and
/// Queens
pub fn find_sliding_moves(
    move_vec: &mut Vec<Move>, pos: &Position, piece: SlidingPiece,
    capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    let mut srcs;
    let target_gen_func: fn(usize, u64) -> u64;
    match piece {
        SlidingPiece::Bishop => {
            srcs = our_pieces.bishop;
            target_gen_func = magics::get_bishop_attacks;
        },
        SlidingPiece::Rook => {
            srcs = our_pieces.rook;
            target_gen_func = magics::get_rook_attacks;
        },
        SlidingPiece::Queen => {
            srcs = our_pieces.queen;
            target_gen_func = magics::get_queen_attacks;
        }
    }
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        let mut targets: u64 = target_gen_func(bt::ilsb(src), pos.data.occ);
        targets &= !our_pieces.any;
        targets &= capture_mask | push_mask;
        if src & pinned_pieces != EMPTY_BB {
            targets &= bt::ray_axis(our_pieces.king, src);
        }
        find_quiet_moves_and_captures(move_vec, pos, targets, src)
    }
}

// Special Moves

/// Move generation function for en passant captures
pub fn find_en_passant_moves(
    move_vec: &mut Vec<Move>, pos: &Position, capture_mask: u64,
    push_mask: u64, pinned_pieces: u64
) {
    let target = pos.data.en_passant_target_sq;
    let captured_pawn = pos.pawn_en_passant_cap();
    if target == EMPTY_BB || (captured_pawn & capture_mask == EMPTY_BB
        && target & push_mask == EMPTY_BB) {
        return
    }
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    let mut srcs = pos.pawn_en_passant_srcs();
    
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        // If pawn is pinned, check capture is along pin axis
        if src & pinned_pieces != EMPTY_BB {
            let pin_mask = bt::ray_axis(our_pieces.king, src);
            if target & pin_mask == EMPTY_BB {
                continue;
            }
        }
        // Check rare en passant case that may occur if the king is on the
        // same rank as the pawns involved in the en passant capture where
        // an en passant capture may reveal a discovered check
        if our_pieces.king & pos.ep_capture_rank() != EMPTY_BB {
            let occ = pos.data.occ & !(src | captured_pawn);
            let king_file_attacks = bt::hyp_quint(
                occ, our_pieces.king, &MAPS.rank
            );
            if king_file_attacks 
                & (their_pieces.rook | their_pieces.queen) != EMPTY_BB {
                continue;
            }
        }
        move_vec.push(Move::new_ep_capture(target, src))
    }
    
}

pub fn find_castling_moves(
    move_vec: &mut Vec<Move>, pos: &Position, unsafe_squares: u64
) {
    let src = pos.our_pieces().king;
    // Kingside castle
    if pos.our_kingside_castle()
        && (pos.kingside_castle_mask() & pos.data.occ) == EMPTY_BB
        && (pos.kingside_castle_mask() & unsafe_squares) == EMPTY_BB 
    {
        move_vec.push(Move::new_short_castle(bt::east_two(src), src))
    }
    // Queenside castle
    if pos.our_queenside_castle()
        && (pos.queenside_castle_mask_free() & pos.data.occ) == EMPTY_BB
        && (pos.queenside_castle_mask_safe() & unsafe_squares) == EMPTY_BB
    {
        move_vec.push(Move::new_long_castle(bt::west_two(src), src))
    }
}

/// Helper function for non-pawn moves, where the capture status is
/// indeterminate. Seperates out the capture moves from the quiet moves and
/// adds them to the move vector
fn find_quiet_moves_and_captures(
    move_vec: &mut Vec<Move>, pos: &Position, targets: u64, src: u64
) {
    let mut capture_targets = targets & pos.their_pieces().any;
    let mut quiet_targets = targets & pos.data.free;
    while capture_targets != EMPTY_BB {
        let target = bt::pop_lsb(&mut capture_targets);
        move_vec.push(Move::new_capture(target, src))
    }
    while quiet_targets != EMPTY_BB {
        let target = bt::pop_lsb(&mut quiet_targets);
        move_vec.push(Move::new_quiet_move(target, src))
    }
}

/// Push all the permutations of a quiet promotion
fn push_promotions(move_vec: &mut Vec<Move>, target: u64, src: u64) {
    move_vec.push(Move::new_queen_promotion(target, src));
    move_vec.push(Move::new_knight_promotion(target, src));
    move_vec.push(Move::new_bishop_promotion(target, src));
    move_vec.push(Move::new_rook_promotion(target, src))
}

/// Push all the permuations of a capture promotion
fn push_promo_captures(move_vec: &mut Vec<Move>, target: u64, src: u64) {
    move_vec.push(Move::new_queen_promo_capture(target, src));
    move_vec.push(Move::new_knight_promo_capture(target, src));
    move_vec.push(Move::new_bishop_promo_capture(target, src));
    move_vec.push(Move::new_rook_promo_capture(target, src))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt::squares_to_bitboard;

    use test_case::test_case;

    fn generate_targets(move_vec: Vec<Move>) -> u64 {
        let mut targets = EMPTY_BB;
        for mv in move_vec {
            targets |= mv.target();
        }
        return targets
    }

    #[test_case(DEFAULT_FEN, 8, vec![16, 17, 18, 19, 20, 21, 22, 23]; "starting")]
    #[test_case(POSITION_2, 4, vec![16, 17, 43, 22]; "position_two")]
    #[test_case(POSITION_3, 3, vec![20, 22, 41]; "position_three")]
    fn test_sgl_push_pawn_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_single_pushes( &mut move_vec, &pos, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets);
    }

    #[test_case(DEFAULT_FEN, 8, vec![24, 25, 26, 27, 28, 29, 30, 31]; "starting")]
    #[test_case(POSITION_2, 2, vec![24, 30]; "position_two")]
    #[test_case(POSITION_3, 2, vec![28, 30]; "position_three")]
    fn test_dbl_push_pawn_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_double_pushes(&mut move_vec, &pos, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 0, vec![]; "position_two")]
    #[test_case(POSITION_3, 0, vec![]; "position_three")]
    fn test_push_lcap_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_left_captures(&mut move_vec, &pos, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![44, 23]; "position_two")]
    #[test_case(POSITION_3, 0, vec![]; "position_three")]
    fn test_push_rcap_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_right_captures(&mut move_vec, &pos, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 4, vec![16, 18, 21, 23]; "starting")]
    #[test_case(POSITION_2, 11, vec![1, 24, 33, 3, 51, 42, 26, 19, 30, 46, 53];
        "position_two")]
    fn test_knight_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_knight_moves(
            &mut move_vec,
            &pos,
            FILLED_BB,
            FILLED_BB, 
            EMPTY_BB,
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![3, 5]; "position_two")]
    fn test_king_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_king_moves(
            &mut move_vec,
            &pos,
            EMPTY_BB
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 11, vec![2, 20, 29, 38, 47, 3, 5, 19, 26, 33, 40];
        "position_two")]
    fn test_bishop_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_sliding_moves(
            &mut move_vec,
            &pos,
            SlidingPiece::Bishop, 
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 5, vec![1, 2, 3, 5, 6]; "position_two")]
    fn test_rook_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_sliding_moves(
            &mut move_vec,
            &pos,
            SlidingPiece::Rook,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 9, vec![19, 20, 22, 23, 29, 37, 45, 30, 39];
        "position_two")]
    fn test_queen_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_sliding_moves(
            &mut move_vec,
            &pos,
            SlidingPiece::Queen,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 0, vec![]; "position_two")]
    fn test_en_passant_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_en_passant_moves(
            &mut move_vec,
            &pos,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB,
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![2, 6]; "position_two")]
    fn test_castling_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let mut move_vec = Vec::new();
        find_castling_moves(
            &mut move_vec,
            &pos,
            EMPTY_BB,
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 20, 0; "starting")]
    #[test_case(POSITION_2, 48, 8; "position_two")]
    #[test_case(POSITION_3, 14, 1; "position_three")]
    #[test_case(POSITION_4, 6, 0; "position_four")]
    fn test_move_gen(
        fen: &str, expected_nodes: i32, expected_captures: i32,
    ) {
        let pos = Position::new_from_fen(fen.to_string());
        let move_vec = find_moves(&pos);
        let mut n_captures = 0;
        for mv in &move_vec {
            if mv.is_capture() {
                n_captures += 1
            }
        }
        assert_eq!(expected_nodes, move_vec.len() as i32, "nodes");
        assert_eq!(expected_captures, n_captures, "captures")
    }
}