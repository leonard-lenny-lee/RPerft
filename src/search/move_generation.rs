use crate::position::{White, Black};
use crate::{common::*, d};
use crate::common::bittools as bt;
use crate::position::{Position, analysis_tools, states::Pos};
use crate::global::maps::Maps;
use strum::IntoEnumIterator;

pub struct Move {
    pub target: u64,
    pub src: u64,
    pub moved_piece: Piece,
    pub promotion_piece: PromotionPiece,
    pub special_move_flag: SpecialMove,
    pub is_capture: bool,
    pub captured_piece: Piece,
}

impl Move {
    pub fn new(
        target_sq: u64, 
        src_sq: u64, 
        moved_piece: Piece, 
        promotion_piece: PromotionPiece, 
        special_move_flag: SpecialMove, 
        pos: &impl Pos
    ) -> Move {
        // Identify which piece has been captured
        let is_capture = pos.their_pieces().any & target_sq != EMPTY_BB;
        let mut captured_piece = Piece::Any;
        if is_capture {
            captured_piece = analysis_tools::get_piece_at(pos, target_sq)
        }
        return Move {
            target: target_sq,
            src: src_sq,
            moved_piece: moved_piece,
            promotion_piece: promotion_piece,
            special_move_flag: special_move_flag,
            is_capture: is_capture,
            captured_piece: captured_piece,
        };
    }
}

/// The master move generation function - generates all legal moves in a
/// position and returns the list of legal moves as a vector of moves
pub fn generate_moves(pos: &impl Pos, maps: &Maps) -> Vec<Move> {
    // Initialise variables
    let mut move_vec: Vec<Move> = Vec::new();
    let (unsafe_squares, checkers) = 
        analysis_tools::find_unsafe_squares_and_checkers_for(&color, pos, maps);
    let pinned_pieces = analysis_tools::get_pinned_pieces_for(pos, &color, maps);
    // Number of pieces placing the king in check
    let n_attackers = checkers.count_ones();
    let mut capture_mask: u64 = FILLED_BB;
    let mut push_mask: u64 = FILLED_BB;
    // If the king is in double check, only king moves to safe squares are valid
    if n_attackers > 1 {
        generate_king_moves(&mut move_vec, pos, maps, unsafe_squares);
        return move_vec;
    }
    if n_attackers == 1 {
        // This means the king is in single check so moves are only legal if
        // 1. It moves the king out of check
        // 2. The attacking piece is captured
        // 3. The attacking piece is blocked, if the piece is a sliding piece
        capture_mask = checkers;
        if analysis_tools::piece_at_is_slider(pos, checkers) {
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
        generate_pawn_moves(
            &mut move_vec, pos, move_type, capture_mask,
            push_mask, pinned_pieces
        )
    }
    generate_knight_moves(
        &mut move_vec, pos, maps, capture_mask,
        push_mask, pinned_pieces
    );
    generate_king_moves(
        &mut move_vec, pos, maps, unsafe_squares
    );
    for piece in SlidingPiece::iter() {
        generate_sliding_moves(
            &mut move_vec, pos, piece, maps, capture_mask, 
            push_mask, pinned_pieces
        )
    }
    // Castling is only allowed if not in check
    if n_attackers == 0 {
        generate_castling_moves(&mut move_vec, pos, unsafe_squares);
    }
    generate_en_passant_moves(
        &mut move_vec, pos, capture_mask, push_mask, maps
    );
    return move_vec;
}

/// Move generation functions. These accept a mutable move vector reference as
/// an argument and pushes legal pawn moves in a position to the move vector

/// General move generation function for pawns in a position.
fn generate_pawn_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    move_type: PawnMove,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64
) {
    let targets;
    let srcs;
    match move_type {
        PawnMove::SinglePush => {
            targets = pos.pawn_sgl_push_targets();
            srcs = pos.pawn_sgl_push_srcs(targets)
        },
        PawnMove::DoublePush => {
            targets = pos.pawn_dbl_push_targets();
            srcs = pos.pawn_dbl_push_srcs(targets)
        },
        PawnMove::CaptureLeft => {
            targets = pos.pawn_lf_cap_targets();
            srcs = pos.pawn_lf_cap_srcs(targets)
        },
        PawnMove::CaptureRight => {
            targets = pos.pawn_rt_cap_targets();
            srcs = pos.pawn_rt_cap_srcs(targets)
        }
    }
    // Only one the push or capture mask should be applied
    let mask: u64;
    if matches!(move_type, PawnMove::SinglePush | PawnMove::DoublePush) {
        mask = push_mask
    } else {
        mask = capture_mask
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
                    PromotionPiece::None,
                    SpecialMove::None,
                    pos
                )
            )
        } else {
            generate_promotions(move_vec, pos, target, src)
        }
    }
}

/// Move generation function for knights
fn generate_knight_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    maps: &Maps,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64,
) {
    let our_pieces = pos.our_pieces();
    let src_vec = bt::forward_scan(our_pieces.knight);
    for src in src_vec {
        let mut targets = maps.get_knight_map(src) & !our_pieces.any;
        // Only allow moves which either capture a checking piece or blocks
        // the check. These masks should be a FILLED_BB when no check.
        targets &= capture_mask | push_mask;
        if src & pinned_pieces != EMPTY_BB {
            // If knight is pinned, there are no legal moves
            continue;
        }
        let target_vec = bt::forward_scan(targets);
        for target in target_vec {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    Piece::Knight,
                    PromotionPiece::None,
                    SpecialMove::None,
                    pos,
                )
            )
        }
    }
}

/// Move generation function for kings
fn generate_king_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    maps: &Maps,
    unsafe_squares: u64
) {
    let our_pieces = pos.our_pieces();
    let src = our_pieces.king;
    let mut targets = maps.get_king_map(src) & !our_pieces.any;
    // Remove unsafe squares i.e. squares attacked by opponent pieces
    // from the available target sqaures for the king
    targets &= !unsafe_squares;
    let target_vec = bt::forward_scan(targets);
    for target in target_vec {
        move_vec.push(
            Move::new(
                target,
                src,
                Piece::King,
                PromotionPiece::None,
                SpecialMove::None,
                pos,
            )
        )
    }
}

/// General move generation function for sliding pieces - Rooks, Bishops and
/// Queens
fn generate_sliding_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    piece: SlidingPiece,
    maps: &Maps,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64,
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
    let src_vec = bt::forward_scan(srcs);
    for src in src_vec {
        let mut targets: u64 = target_gen_func(pos.position().occ, src, maps);
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
        let target_vec = bt::forward_scan(targets);
        for target in target_vec {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    moved_piece,
                    PromotionPiece::None,
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
fn generate_promotions(
    move_vec: &mut Vec<Move>, 
    pos: &impl Pos, 
    target: u64, 
    src: u64
) {
    for piece in PromotionPiece::iterator() {
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
fn generate_en_passant_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    capture_mask: u64,
    push_mask: u64,
    maps: &Maps
) {
    let position = pos.position();
    let target = position.en_passant_target_sq & push_mask;
    let captured_pawn = pos.pawn_en_passant_cap();
    if target == EMPTY_BB || captured_pawn & capture_mask == EMPTY_BB {
        return
    }
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    
    for src in bt::forward_scan(pos.pawn_en_passant_srcs()) {
        // Check rare en passant case that may occur if the king is on the
        // same rank as the pawns involved in the en passant capture where
        // an en passant capture may reveal a discovered check
        if our_pieces.king & pos.ep_capture_rank() != EMPTY_BB {
            let occ = position.occ ^ (src | captured_pawn);
            let king_file_attacks = bt::hyp_quint(
                occ, our_pieces.king, &maps.file
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
                PromotionPiece::None,
                SpecialMove::EnPassant,
                pos,
            )
        )
    }
    
}

fn generate_castling_moves(
    move_vec: &mut Vec<Move>,
    pos: &impl Pos,
    unsafe_squares: u64
) {
    let src = pos.our_pieces().king;
    let position = pos.position();
    // Kingside castle
    if pos.ksc() && pos.kscm() & position.occ & unsafe_squares == EMPTY_BB {
        move_vec.push(
            Move::new(
                bt::east_two(src),
                src,
                Piece::King,
                PromotionPiece::None,
                SpecialMove::Castling,
                pos
            )
        )
    }
    if pos.qsc() && pos.qscm() & position.occ & unsafe_squares == EMPTY_BB {
        move_vec.push(
            Move::new(
                bt::west_two(src),
                src,
                Piece::King,
                PromotionPiece::None,
                SpecialMove::Castling,
                pos
            )
        )
    }
}

// TODO Refactor
pub fn apply_move(pos: &impl Pos, mv: &Move) -> dyn Pos {
    let position = pos.position();
    if position.white_to_move {
        return White{pos: *position}
    } else {
        return Black{pos: *position}
    }

    // // Common operations for all moves
    // let move_mask = mv.src | mv.target;
    // pos.free |= mv.src; // Source squares must be free now
    // pos.occ &= !mv.src;
    // pos.free &= !mv.target; // Target sqaures must be occupied
    // pos.occ |= mv.target;
    // // Our bitboards must be flipped at target and source
    // our_pieces.xor_assign(d!(mv.moved_piece), move_mask); 
    // our_pieces.any ^= move_mask;
    // // Free the squares on the their bitboards if the piece is a capture
    // if mv.is_capture {
    //     their_pieces.xor_assign(d!(mv.captured_piece), mv.target);
    //     their_pieces.any ^= mv.target;
    //     if matches!(mv.captured_piece, Piece::Rook) && mv.target & ROOK_START != 0 {
    //         // If a rook on its starting square is captured, always set the
    //         // castling rights as false.
    //         match mv.target {
    //             WQROOK => pos.w_queenside_castle = false,
    //             WKROOK => pos.w_kingside_castle = false,
    //             BQROOK => pos.b_queenside_castle = false,
    //             BKROOK => pos.b_kingside_castle = false,
    //             _ => ()
    //         }
    //     }
    // }
    // // Similarly, if a rook has been moved from its starting square, always
    // // set the castling rights as false
    // if matches!(mv.moved_piece, Piece::Rook) && mv.src & ROOK_START != 0 {
    //     match mv.src {
    //         WQROOK => pos.w_queenside_castle = false,
    //         WKROOK => pos.w_kingside_castle = false,
    //         BQROOK => pos.b_queenside_castle = false,
    //         BKROOK => pos.b_kingside_castle = false,
    //         _ => ()
    //     }
    // }
    // // Set en passant target sq to empty, this will be set to the relevant
    // // value for dbl pawn pushes later
    // pos.en_passant_target_sq = EMPTY_BB;
    // // Reset the halfmove clock if a pawn is moved or a capture has taken
    // // place. Else, increment the halfmove clock
    // if mv.is_capture || matches!(mv.moved_piece, Piece::Pawn) {
    //     pos.halfmove_clock = 0;
    // } else {
    //     pos.halfmove_clock += 1;
    // }
    // // Increment the fullmove clock if black has moved
    // if !pos.white_to_move {
    //     pos.fullmove_clock += 1;
    // }
    // match mv.special_move_flag {
    //     SpecialMove::None => (),
    //     SpecialMove::Promotion => {
    //         // Set target square on promotion piece bitboard
    //         our_pieces.bit_or_assign(d!(mv.promotion_piece), mv.target);
    //         // Unset the pawn from our pawn bitboard
    //         our_pieces.pawn ^= mv.target;
    //     },
    //     SpecialMove::Castling => {
    //         assert!(matches!(mv.moved_piece, Piece::King));
    //         // For castling moves, we also need the update the rook
    //         // bitboard and the our universal bitboard
    //         // Calculate if kingside or queenside castle
    //         let rook_castle_mask: u64;
    //         if mv.target.trailing_zeros() % 8 == 6 {
    //             // For kingside castle, the rook has transported from a
    //             // position one east of the target square to one west
    //             rook_castle_mask = mv.target << 1 | mv.target >> 1;
    //         } else {
    //             // For the queenside castle, the rook has transported from
    //             // a position 2 squares west of the target square to the
    //             // position 1 east of the target sqaure
    //             assert!(mv.target.trailing_zeros() % 8 == 2);
    //             rook_castle_mask = mv.target << 1 | mv.target >> 2;
    //         }
    //         our_pieces.rook ^= rook_castle_mask;
    //         our_pieces.any ^= rook_castle_mask;
    //         // Disallow any more castling moves if a castle has occurred
    //         if pos.white_to_move {
    //             pos.w_kingside_castle = false;
    //             pos.w_queenside_castle = false;
    //         } else {
    //             pos.b_kingside_castle = false;
    //             pos.b_queenside_castle = false;
    //         }
    //     },
    //     SpecialMove::EnPassant => {
    //         assert!(pos.en_passant_target_sq != 0);
    //         let ep_capture_sq;
    //         if pos.white_to_move {
    //             // If white made the en passant capture, then the square at
    //             // which the capture takes place is on square south of the
    //             // target square
    //             ep_capture_sq = bt::south_one(mv.target)
    //         } else {
    //             // Opposite for black
    //             ep_capture_sq = bt::north_one(mv.target)
    //         }
    //         // Reflect the capture on the opponent bitboards
    //         their_pieces.any ^= ep_capture_sq;
    //         their_pieces.pawn ^= ep_capture_sq;
    //     },
    //     SpecialMove::DoublePush => {
    //         // Set enpassant square if the move was a double push
    //         if pos.white_to_move {
    //             // If white made the double pawn push, then the ep target
    //             // square must be one square north of the source
    //             pos.en_passant_target_sq = bt::north_one(mv.src)
    //         } else {
    //             // Vice versa for black
    //             pos.en_passant_target_sq = bt::south_one(mv.src)
    //         }
    //     }
    // }
    // // Change the turn
    // pos.white_to_move = !pos.white_to_move;
    // return pos
}

#[cfg(test)]
mod tests;