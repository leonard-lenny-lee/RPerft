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
    for move_type in PawnMove::iter() {
        find_pawn_moves(
            &mut move_vec, pos, move_type, capture_mask,
            push_mask, pinned_pieces
        )
    }
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
/// an argument and pushes legal pawn moves in a position to the move vector

/// General move generation function for pawns in a position.
pub fn find_pawn_moves(
    move_vec: &mut Vec<Move>, pos: &Position, move_type: PawnMove,
    capture_mask: u64, push_mask: u64, pinned_pieces: u64
) {
    let mut targets;
    let mut srcs;
    let mut special_move_flag = SpecialMove::None;
    match move_type {
        PawnMove::SinglePush => {
            targets = pos.pawn_sgl_push_targets() & push_mask;
            srcs = pos.pawn_sgl_push_srcs(targets)
        },
        PawnMove::DoublePush => {
            targets = pos.pawn_dbl_push_targets() & push_mask;
            srcs = pos.pawn_dbl_push_srcs(targets);
            special_move_flag = SpecialMove::DoublePush
        },
        PawnMove::CaptureLeft => {
            targets = pos.pawn_lcap_targets() & capture_mask;
            srcs = pos.pawn_lcap_srcs(targets)
        },
        PawnMove::CaptureRight => {
            targets = pos.pawn_rcap_targets() & capture_mask;
            srcs = pos.pawn_rcap_srcs(targets)
        }
    }
    while targets != EMPTY_BB {
        let target = bt::pop_lsb(&mut targets);
        let src = bt::pop_lsb(&mut srcs);
        // Check if the pawn is pinned, only allow moves towards/away from king
        if src & pinned_pieces != EMPTY_BB {
            let pin_mask = bt::ray_axis(
                pos.our_pieces().king,
                src
            );
            if target & pin_mask == EMPTY_BB {
                continue;
            }
        }       
        // Check if the target is a promotion square
        if target & pos.promotion_rank() == EMPTY_BB {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    Piece::Pawn,
                    Promotion::None,
                    special_move_flag,
                    pos
                )
            )
        } else {
            find_promotions(move_vec, pos, target, src)
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
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        let mut targets = MAPS.get_knight_map(src) & !our_pieces.any;
        // Only allow moves which either capture a checking piece or blocks
        // the check. These masks should be a FILLED_BB when no check.
        targets &= capture_mask | push_mask;
        if src & pinned_pieces != EMPTY_BB {
            // If knight is pinned, there are no legal moves
            continue;
        }
        while targets != EMPTY_BB {
            let target = bt::pop_lsb(&mut targets);
            move_vec.push(
                Move::new(
                    target,
                    src,
                    Piece::Knight,
                    Promotion::None,
                    SpecialMove::None,
                    pos,
                )
            )
        }
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
    while targets != EMPTY_BB {
        let target = bt::pop_lsb(&mut targets);
        move_vec.push(
            Move::new(
                target,
                src,
                Piece::King,
                Promotion::None,
                SpecialMove::None,
                pos,
            )
        )
    }
}

/// General move generation function for sliding pieces - Rooks, Bishops and
/// Queens
pub fn find_sliding_moves(
    move_vec: &mut Vec<Move>, pos: &Position, piece: SlidingPiece,
    capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    let mut srcs;
    let moved_piece;
    let target_gen_func: fn(u64, u64) -> u64;
    match piece {
        SlidingPiece::Bishop => {
            srcs = our_pieces.bishop;
            target_gen_func = bt::da_hyp_quint;
            moved_piece = Piece::Bishop;
        },
        SlidingPiece::Rook => {
            srcs = our_pieces.rook;
            target_gen_func = bt::hv_hyp_quint;
            moved_piece = Piece::Rook;
        },
        SlidingPiece::Queen => {
            srcs = our_pieces.queen;
            target_gen_func = bt::all_hyp_quint;
            moved_piece = Piece::Queen;
        }
    }
    while srcs != EMPTY_BB {
        let src = bt::pop_lsb(&mut srcs);
        let mut targets: u64 = target_gen_func(pos.data.occ, src);
        targets &= !our_pieces.any;
        targets &= capture_mask | push_mask;
        // If piece is pinned, it can only move the direction directly to 
        // or from the king
        if pinned_pieces & src != EMPTY_BB {
            let pin_mask = bt::ray_axis(
                our_pieces.king, src
            );
            targets &= pin_mask;
        }
        while targets != EMPTY_BB {
            let target = bt::pop_lsb(&mut targets);
            move_vec.push(
                Move::new(
                    target,
                    src,
                    moved_piece,
                    Promotion::None,
                    SpecialMove::None,
                    pos,
                )
            )
        }
    }
}

// Special Moves

/// Move generation function for promotions, this is called by the general
/// pawn generation function if a target square is on the promotion rank
fn find_promotions(
    move_vec: &mut Vec<Move>, pos: &Position, target: u64, src: u64
) {
    for piece in Promotion::iterator() {
        move_vec.push(
            Move::new(
                target,
                src,
                Piece::Pawn,
                piece,
                SpecialMove::Promotion,
                pos,                    
            )
        )
    }
}

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
        move_vec.push(
            Move::new(
                target,
                src,
                Piece::Pawn,
                Promotion::None,
                SpecialMove::EnPassant,
                pos,
            )
        )
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
        move_vec.push(
            Move::new(
                bt::east_two(src),
                src,
                Piece::King,
                Promotion::None,
                SpecialMove::Castling,
                pos
            )
        )
    }
    // Queenside castle
    if pos.our_queenside_castle()
        && (pos.queenside_castle_mask_free() & pos.data.occ) == EMPTY_BB
        && (pos.queenside_castle_mask_safe() & unsafe_squares) == EMPTY_BB
    {
        move_vec.push(
            Move::new(
                bt::west_two(src),
                src,
                Piece::King,
                Promotion::None,
                SpecialMove::Castling,
                pos
            )
        )
    }
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
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::SinglePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
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
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::DoublePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
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
        find_pawn_moves(
            &mut move_vec,
            &pos, 
            PawnMove::CaptureLeft,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
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
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::CaptureRight,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
        assert_eq!(expected_nodes, move_vec.len() as i32);
        let targets = generate_targets(move_vec);
        let expected_targets = squares_to_bitboard(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 4, vec![16, 18, 21, 23]; "starting")]
    #[test_case(POSITION_2, 11, vec![1, 24, 33, 3, 51, 42, 26, 19, 30, 46, 53]; "position_two")]
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
    #[test_case(POSITION_2, 11, vec![2, 20, 29, 38, 47, 3, 5, 19, 26, 33, 40]; "position_two")]
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
    #[test_case(POSITION_2, 9, vec![19, 20, 22, 23, 29, 37, 45, 30, 39]; "position_two")]
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