use super::*;
use movelist::MoveList;
use position::Position;
use std::iter::zip;

/// Generate a MoveList of all legal moves
pub fn find_moves(pos: &Position) -> MoveList {
    // Initialise variables
    let mut move_list: MoveList = MoveList::new();
    let unsafe_squares = pos.unsafe_squares();
    let checkers = pos.find_checkers();
    let pinned_pieces = pos.pinned_pieces();

    // Number of pieces placing the king in check
    let n_checkers = checkers.pop_count();
    let mut capture_mask = FILLED_BB;
    let mut push_mask = FILLED_BB;

    match n_checkers.cmp(&1) {
        std::cmp::Ordering::Less => (),
        std::cmp::Ordering::Equal => {
            // In single check, the attacking piece can also be captured
            // so set the location of the checker as the capture mask
            capture_mask = checkers;
            if pos.their_piece_at_is_slider(checkers) {
                // If the attacker is a sliding piece, then check can be
                // blocked by another piece moving into the intervening squares
                push_mask = pos.our_pieces().king.connect_squares(checkers)
            } else {
                // It can only be captured so give no options to block
                push_mask = EMPTY_BB
            }
        }
        std::cmp::Ordering::Greater => {
            // If the king is in double check, only king moves are valid
            find_king_moves(pos, &mut move_list, unsafe_squares);
            return move_list;
        }
    }

    // Add all moves to the move list
    find_single_pushes(pos, &mut move_list, push_mask, pinned_pieces);
    find_double_pushes(pos, &mut move_list, push_mask, pinned_pieces);
    find_pawn_captures(pos, &mut move_list, capture_mask, pinned_pieces);
    find_knight_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    find_king_moves(pos, &mut move_list, unsafe_squares);
    find_sliding_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    if n_checkers == 0 {
        find_castling_moves(pos, &mut move_list, unsafe_squares);
    }
    find_en_passant_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    return move_list;
}

/// Variation of the find moves function to find only captures
pub fn find_captures(pos: &Position) -> MoveList {
    debug_assert!(pos.find_checkers().pop_count() == 0);
    let mut move_list = MoveList::new();
    let their_pieces = pos.their_pieces();

    let unsafe_squares = pos.unsafe_squares() | !their_pieces.any;
    let pinned_pieces = pos.pinned_pieces();
    let capture_mask = their_pieces.any;
    let push_mask = their_pieces.any;

    find_pawn_captures(pos, &mut move_list, capture_mask, pinned_pieces);
    find_knight_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    find_king_moves(pos, &mut move_list, unsafe_squares);
    find_sliding_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    find_en_passant_moves(pos, &mut move_list, capture_mask, push_mask, pinned_pieces);
    return move_list;
}

/// Find only check evasions in a position. Panics if used on a position where
/// they are not in check
pub fn find_check_evasions(pos: &Position, checkers: BB) -> MoveList {
    let n_checkers = checkers.pop_count();
    debug_assert!(n_checkers >= 1);
    let mut move_list = MoveList::new();
    let unsafe_squares = pos.unsafe_squares();
    let pinned_pieces = pos.pinned_pieces();
    if n_checkers > 1 {
        // If in double check, only king moves are valid
        find_king_moves(pos, &mut move_list, unsafe_squares);
        return move_list;
    }
    // n_checkers must be 1
    let push_mask = if pos.their_piece_at_is_slider(checkers) {
        pos.our_pieces().king.connect_squares(checkers)
    } else {
        EMPTY_BB
    };
    find_single_pushes(pos, &mut move_list, push_mask, pinned_pieces);
    find_double_pushes(pos, &mut move_list, push_mask, pinned_pieces);
    find_pawn_captures(pos, &mut move_list, checkers, pinned_pieces);
    find_knight_moves(pos, &mut move_list, checkers, push_mask, pinned_pieces);
    find_king_moves(pos, &mut move_list, unsafe_squares);
    find_sliding_moves(pos, &mut move_list, checkers, push_mask, pinned_pieces);
    find_en_passant_moves(pos, &mut move_list, checkers, push_mask, pinned_pieces);
    return move_list;
}

// Move generation methods to push valid moves onto the move list

#[inline]
/// Move generation function to find all pawn single pushes in a position.
fn find_single_pushes(pos: &Position, move_list: &mut MoveList, push_mask: BB, pinned_pieces: BB) {
    let our_pieces = pos.our_pieces();
    let free_pawns = our_pieces.pawn & !pinned_pieces;
    let pinned_pawns = our_pieces.pawn & pinned_pieces;
    let target_filter = pos.data.free & push_mask;

    let targets = (pos.pawn_sgl_push(free_pawns) & target_filter)
        | (pos.pawn_sgl_push(pinned_pawns) & target_filter & our_pieces.king.lu_file_mask());

    let promotion_rank = pos.target_promotion_rank();
    let promo_targets = targets & promotion_rank;
    let promo_srcs = pos.pawn_sgl_push_srcs(promo_targets);
    // Normal pawns
    for (target, src) in std::iter::zip(promo_targets, promo_srcs) {
        move_list.add_promotions(target, src)
    }
    let quiet_targets = targets & !promotion_rank;
    let quiet_srcs = pos.pawn_sgl_push_srcs(quiet_targets);
    for (target, src) in std::iter::zip(quiet_targets, quiet_srcs) {
        move_list.add_quiet_move(target, src)
    }
}

#[inline]
/// Move generation function to find all pawn double pushes in a position
fn find_double_pushes(pos: &Position, move_list: &mut MoveList, push_mask: BB, pinned_pieces: BB) {
    let our_pieces = pos.our_pieces();

    let sgl_push = pos.pawn_sgl_push(our_pieces.pawn & pos.pawn_start_rank()) & pos.data.free;
    let dbl_push = pos.pawn_sgl_push(sgl_push) & pos.data.free & push_mask;
    let pinned_targets = pos.pawn_dbl_push(pinned_pieces);

    let targets =
        (dbl_push & !pinned_targets) | (dbl_push & pinned_targets & our_pieces.king.lu_file_mask());
    let srcs = pos.pawn_dbl_push_srcs(targets);
    for (target, src) in zip(targets, srcs) {
        move_list.add_double_pawn_push(target, src)
    }
}

#[inline]
fn find_pawn_captures(
    pos: &Position,
    move_list: &mut MoveList,
    capture_mask: BB,
    pinned_pieces: BB,
) {
    let our_pieces = pos.our_pieces();
    let target_filter = pos.their_pieces().any & capture_mask;

    let free_pawns = our_pieces.pawn & !pinned_pieces;
    let pinned_pawns = our_pieces.pawn & pinned_pieces;
    let promotion_rank = pos.target_promotion_rank();

    // Left captures
    let left_targets = (pos.pawn_left_capture(free_pawns) & target_filter)
        | (pos.pawn_left_capture(pinned_pawns)
            & target_filter
            & pos.pawn_left_capture_pin_mask(our_pieces.king));

    let left_normal_targets = left_targets & !promotion_rank;
    let left_normal_srcs = pos.pawn_lcap_srcs(left_normal_targets);
    for (target, src) in zip(left_normal_targets, left_normal_srcs) {
        move_list.add_capture(target, src);
    }
    let left_promo_targets = left_targets & promotion_rank;
    let left_promo_srcs = pos.pawn_lcap_srcs(left_promo_targets);
    for (target, src) in zip(left_promo_targets, left_promo_srcs) {
        move_list.add_promotion_captures(target, src)
    }

    // Right captures
    let right_targets = (pos.pawn_right_capture(free_pawns) & target_filter)
        | (pos.pawn_right_capture(pinned_pawns)
            & target_filter
            & pos.pawn_right_capture_pin_mask(our_pieces.king));

    let right_normal_targets = right_targets & !promotion_rank;
    let right_normal_srcs = pos.pawn_rcap_srcs(right_normal_targets);
    for (target, src) in zip(right_normal_targets, right_normal_srcs) {
        move_list.add_capture(target, src);
    }
    let right_promo_targets = right_targets & promotion_rank;
    let right_promo_srcs = pos.pawn_rcap_srcs(right_promo_targets);
    for (target, src) in zip(right_promo_targets, right_promo_srcs) {
        move_list.add_promotion_captures(target, src)
    }
}

#[inline]
/// Move generation function for knights
fn find_knight_moves(
    pos: &Position,
    move_list: &mut MoveList,
    capture_mask: BB,
    push_mask: BB,
    pinned_pieces: BB,
) {
    let our_pieces = pos.our_pieces();
    // Only allow moves which either capture a checking piece or blocks
    // the check. These masks should be a FILLED_BB when no check.
    let target_filter = !our_pieces.any & (capture_mask | push_mask);
    // Filter out knights which are pinned - pinned knights have
    // no legal moves
    let srcs = our_pieces.knight & !pinned_pieces;
    for src in srcs {
        let targets = src.lu_knight_attacks() & target_filter;
        find_quiet_moves_and_captures(pos, move_list, targets, src)
    }
}

#[inline]
/// Append onto a move list the king moves
fn find_king_moves(pos: &Position, move_list: &mut MoveList, unsafe_squares: BB) {
    let our_pieces = pos.our_pieces();
    let src = our_pieces.king;
    // Remove unsafe squares i.e. squares attacked by opponent pieces
    // from the available target sqaures for the king
    let targets = src.lu_king_attacks() & !our_pieces.any & !unsafe_squares;
    find_quiet_moves_and_captures(pos, move_list, targets, src)
}

#[inline]
fn find_sliding_moves(
    pos: &Position,
    move_list: &mut MoveList,
    capture_mask: BB,
    push_mask: BB,
    pinned_pieces: BB,
) {
    let our_pieces = pos.our_pieces();
    let rook_movers = our_pieces.rook | our_pieces.queen;
    let bishop_movers = our_pieces.bishop | our_pieces.queen;
    let filter = !our_pieces.any & (capture_mask | push_mask);

    for src in rook_movers & !pinned_pieces {
        let targets = src.lu_rook_attacks(pos.data.occ) & filter;
        find_quiet_moves_and_captures(pos, move_list, targets, src);
    }
    for src in rook_movers & pinned_pieces {
        let targets = src.lu_rook_attacks(pos.data.occ) & filter & our_pieces.king.common_axis(src);
        find_quiet_moves_and_captures(pos, move_list, targets, src)
    }
    for src in bishop_movers & !pinned_pieces {
        let targets = src.lu_bishop_attacks(pos.data.occ) & filter;
        find_quiet_moves_and_captures(pos, move_list, targets, src);
    }
    for src in bishop_movers & pinned_pieces {
        let targets =
            src.lu_bishop_attacks(pos.data.occ) & filter & our_pieces.king.common_axis(src);
        find_quiet_moves_and_captures(pos, move_list, targets, src)
    }
}

// Special Moves

#[inline]
/// Move generation function for en passant captures
fn find_en_passant_moves(
    pos: &Position,
    move_list: &mut MoveList,
    capture_mask: BB,
    push_mask: BB,
    pinned_pieces: BB,
) {
    let target = pos.data.en_passant_target_sq;
    let captured_pawn = pos.pawn_en_passant_cap();
    if target == EMPTY_BB
        || (captured_pawn & capture_mask == EMPTY_BB && target & push_mask == EMPTY_BB)
    {
        return;
    }
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    let mut srcs = pos.pawn_en_passant_srcs();

    while srcs.is_any() {
        let src = srcs.pop_ls1b();
        // If pawn is pinned, check capture is along pin axis
        if (src & pinned_pieces).is_any() {
            let pin_mask = our_pieces.king.common_axis(src);
            if target & pin_mask == EMPTY_BB {
                continue;
            }
        }
        // Check rare en passant case that may occur if the king is on the
        // same rank as the pawns involved in the en passant capture where
        // an en passant capture may reveal a discovered check
        if (our_pieces.king & pos.ep_capture_rank()).is_any() {
            let occ = pos.data.occ & !(src | captured_pawn);
            if (our_pieces.king.hyp_quint(occ, Axis::Rank)
                & (their_pieces.rook | their_pieces.queen))
                .is_any()
            {
                continue;
            }
        }
        move_list.add_en_passant_capture(target, src);
    }
}

#[inline]
fn find_castling_moves(pos: &Position, move_list: &mut MoveList, unsafe_squares: BB) {
    let src = pos.our_pieces().king;
    // Kingside castle
    if pos.our_kingside_castle()
        && (pos.kingside_castle_mask() & pos.data.occ).is_empty()
        && (pos.kingside_castle_mask() & unsafe_squares).is_empty()
    {
        move_list.add_short_castle(src.east_two(), src)
    }
    // Queenside castle
    if pos.our_queenside_castle()
        && (pos.queenside_castle_mask_free() & pos.data.occ).is_empty()
        && (pos.queenside_castle_mask_safe() & unsafe_squares).is_empty()
    {
        move_list.add_long_castle(src.west_two(), src)
    }
}

#[inline]
/// Helper function for non-pawn moves, where the capture status is
/// indeterminate. Seperates out the capture moves from the quiet moves and
/// adds them to the move vector
fn find_quiet_moves_and_captures(pos: &Position, move_list: &mut MoveList, targets: BB, src: BB) {
    let capture_targets = targets & pos.their_pieces().any;
    let quiet_targets = targets & pos.data.free;
    for target in capture_targets {
        move_list.add_capture(target, src);
    }
    for target in quiet_targets {
        move_list.add_quiet_move(target, src);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    fn generate_targets(move_list: MoveList) -> BB {
        let mut targets = EMPTY_BB;
        for mv in move_list.iter() {
            targets |= mv.target();
        }
        return targets;
    }

    #[test_case(DEFAULT_FEN, 8, vec![16, 17, 18, 19, 20, 21, 22, 23]; "starting")]
    #[test_case(POSITION_2, 4, vec![16, 17, 43, 22]; "position_two")]
    #[test_case(POSITION_3, 3, vec![20, 22, 41]; "position_three")]
    fn test_sgl_push_pawn_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_single_pushes(&pos, &mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets);
    }

    #[test_case(DEFAULT_FEN, 8, vec![24, 25, 26, 27, 28, 29, 30, 31]; "starting")]
    #[test_case(POSITION_2, 2, vec![24, 30]; "position_two")]
    #[test_case(POSITION_3, 2, vec![28, 30]; "position_three")]
    fn test_dbl_push_pawn_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_double_pushes(&pos, &mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![44, 23]; "position_two")]
    #[test_case(POSITION_3, 0, vec![]; "position_three")]
    fn test_pawn_cap_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_pawn_captures(&pos, &mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 4, vec![16, 18, 21, 23]; "starting")]
    #[test_case(POSITION_2, 11, vec![1, 24, 33, 3, 51, 42, 26, 19, 30, 46, 53];
        "position_two")]
    fn test_knight_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_knight_moves(&pos, &mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![3, 5]; "position_two")]
    fn test_king_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_king_moves(&pos, &mut move_list, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 0, vec![]; "position_two")]
    fn test_en_passant_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_en_passant_moves(&pos, &mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![2, 6]; "position_two")]
    fn test_castling_move_gen(fen: &str, expected_nodes: i32, expected_targets: Vec<usize>) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        find_castling_moves(&pos, &mut move_list, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 20, 0; "starting")]
    #[test_case(POSITION_2, 48, 8; "position_two")]
    #[test_case(POSITION_3, 14, 1; "position_three")]
    #[test_case(POSITION_4, 6, 0; "position_four")]
    fn test_move_gen(fen: &str, expected_nodes: i32, expected_captures: i32) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let move_list = find_moves(&pos);
        let mut n_captures = 0;
        for mv in move_list.iter() {
            if mv.is_capture() {
                n_captures += 1
            }
        }
        assert_eq!(expected_nodes, move_list.len() as i32, "nodes");
        assert_eq!(expected_captures, n_captures, "captures")
    }

    #[test_case(POSITION_2, 8; "position_two")]
    #[test_case(POSITION_3, 1; "position_three")]
    fn test_find_captures(fen: &str, expected_captures: i32) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let move_list = find_captures(&pos);
        assert_eq!(move_list.len() as i32, expected_captures);
        let mut n_captures = 0;
        for mv in move_list.iter() {
            if mv.is_capture() {
                n_captures += 1
            }
        }
        assert_eq!(n_captures, expected_captures);
    }

    #[test]
    fn test_find_check_evasions() {
        let pos = Position::from_fen("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_string()).unwrap();
        let checkers = pos.find_checkers();
        let move_list = find_check_evasions(&pos, checkers);
        assert_eq!(move_list.len(), 11)
    }
}
