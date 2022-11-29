use crate::{common::*, disc};
use crate::common::bittools as bt;
use crate::position::{Position, analysis_tools};
use crate::global::maps::Maps;
use strum::IntoEnumIterator;

pub mod apply_move;

pub struct Move {
    pub target: u64,
    pub src: u64,
    pub moved_piece: Piece,
    pub promotion_piece: Promotion,
    pub special_move_flag: SpecialMove,
    pub is_capture: bool,
    pub captured_piece: Piece,
}

impl Move {
    pub fn new(
        target_sq: u64, src_sq: u64, moved_piece: Piece, 
        promotion_piece: Promotion, special_move_flag: SpecialMove, 
        pos: &Position
    ) -> Move {
        // Identify which piece has been captured
        let is_capture = pos.their_pieces().any & target_sq != EMPTY_BB;
        let mut captured_piece = Piece::Any;
        if is_capture {
            captured_piece = analysis_tools::get_their_piece_at(
                pos, target_sq
            )
        }
        return Move {
            target: target_sq,
            src: src_sq,
            moved_piece,
            promotion_piece,
            special_move_flag,
            is_capture,
            captured_piece,
        };
    }
}

/// The master move finding function - finds all legal moves in a
/// position and returns the list of legal moves as a vector of moves
pub fn find_moves(pos: &Position, maps: &Maps) -> Vec<Move> {
    // Initialise variables
    let mut move_vec: Vec<Move> = Vec::new();
    let unsafe_squares = analysis_tools::find_unsafe_squares(pos, maps);
    let checkers = analysis_tools::find_checkers(pos, maps);
    let pinned_pieces = analysis_tools::get_pinned_pieces_for(pos, maps);
    // Number of pieces placing the king in check
    let n_checkers = checkers.count_ones();
    let mut capture_mask: u64 = FILLED_BB;
    let mut push_mask: u64 = FILLED_BB;
    // If the king is in double check, only king moves to safe squares are valid
    if n_checkers > 1 {
        find_king_moves(&mut move_vec, pos, maps, unsafe_squares);
        return move_vec;
    }
    if n_checkers == 1 {
        // This means the king is in single check so moves are only legal if
        // 1. It moves the king out of check
        // 2. The attacking piece is captured
        // 3. The attacking piece is blocked, if the piece is a sliding piece
        capture_mask = checkers;
        if analysis_tools::their_piece_at_is_slider(pos, checkers) {
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
        &mut move_vec, pos, maps, capture_mask,
        push_mask, pinned_pieces
    );
    find_king_moves(
        &mut move_vec, pos, maps, unsafe_squares
    );
    for piece in SlidingPiece::iter() {
        find_sliding_moves(
            &mut move_vec, pos, piece, maps, capture_mask, 
            push_mask, pinned_pieces
        )
    }
    // Castling is only allowed if not in check
    if n_checkers == 0 {
        find_castling_moves(&mut move_vec, pos, unsafe_squares);
    }
    find_en_passant_moves(
        &mut move_vec, pos, capture_mask, push_mask, maps, pinned_pieces
    );
    return move_vec;
}

/// Move generation functions. These accept a mutable move vector reference as
/// an argument and pushes legal pawn moves in a position to the move vector

/// General move generation function for pawns in a position.
fn find_pawn_moves(
    move_vec: &mut Vec<Move>, pos: &Position, move_type: PawnMove,
    capture_mask: u64, push_mask: u64, pinned_pieces: u64
) {
    let targets;
    let srcs;
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
    let target_vec = bt::forward_scan(targets);
    let src_vec = bt::forward_scan(srcs);
    assert_eq!(target_vec.len(), src_vec.len());
    for i in 0..target_vec.len() {
        let src = src_vec[i];
        let target = target_vec[i];
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
fn find_knight_moves(
    move_vec: &mut Vec<Move>, pos: &Position, maps: &Maps, 
    capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    for src in bt::forward_scan(our_pieces.knight) {
        let mut targets = maps.get_knight_map(src) & !our_pieces.any;
        // Only allow moves which either capture a checking piece or blocks
        // the check. These masks should be a FILLED_BB when no check.
        targets &= capture_mask | push_mask;
        if src & pinned_pieces != EMPTY_BB {
            // If knight is pinned, there are no legal moves
            continue;
        }
        for target in bt::forward_scan(targets) {
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
fn find_king_moves(
    move_vec: &mut Vec<Move>, pos: &Position, maps: &Maps,
    unsafe_squares: u64
) {
    let our_pieces = pos.our_pieces();
    let src = our_pieces.king;
    let mut targets = maps.get_king_map(src) & !our_pieces.any;
    // Remove unsafe squares i.e. squares attacked by opponent pieces
    // from the available target sqaures for the king
    targets &= !unsafe_squares;
    for target in bt::forward_scan(targets) {
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
fn find_sliding_moves(
    move_vec: &mut Vec<Move>, pos: &Position, piece: SlidingPiece,
    maps: &Maps, capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    let srcs;
    let moved_piece;
    let target_gen_func: fn(u64, u64, &Maps) -> u64;
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
    for src in bt::forward_scan(srcs) {
        let mut targets: u64 = target_gen_func(pos.data.occ, src, maps);
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
        for target in bt::forward_scan(targets) {
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
fn find_en_passant_moves(
    move_vec: &mut Vec<Move>, pos: &Position, capture_mask: u64,
    push_mask: u64, maps: &Maps, pinned_pieces: u64
) {
    let target = pos.data.en_passant_target_sq;
    let captured_pawn = pos.pawn_en_passant_cap();
    if target == EMPTY_BB || (captured_pawn & capture_mask == EMPTY_BB
        && target & push_mask == EMPTY_BB) {
        return
    }
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    
    for src in bt::forward_scan(pos.pawn_en_passant_srcs()) {
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
                occ, our_pieces.king, &maps.rank
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

fn find_castling_moves(
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
mod tests;