use crate::{common::*, d};
use crate::common::bittools as bt;
use crate::position::analysis_tools;
use crate::position::Position;
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
        moved_piece: &Piece, 
        promotion_piece: PromotionPiece, 
        special_move_flag: SpecialMove, 
        position: &Position
    ) -> Move {
        let their_pieces;
        if position.white_to_move {
            their_pieces = &position.b_pieces;
        } else {
            their_pieces = &position.w_pieces;
        }
        // Identify which piece has been captured
        let is_capture = their_pieces[d!(Piece::Any)] & target_sq != 0;
        let mut captured_piece = Piece::Any;
        if is_capture {
            for piece in Piece::iter_pieces() {
                if their_pieces[d!(piece)] & target_sq != EMPTY_BB {
                    captured_piece = piece;
                    break;
                }
            }
        }
        return Move {
            target: target_sq,
            src: src_sq,
            moved_piece: *moved_piece,
            promotion_piece: promotion_piece,
            special_move_flag: special_move_flag,
            is_capture: is_capture,
            captured_piece: captured_piece,
        };
    }
}

// Move generation functions

fn generate_pawn_moves(
    move_vec: &mut Vec<Move>,
    pos: &Position,
    move_type: PawnMove,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64
) {
    let target_gen_funcs: [fn(&Position) -> u64; 4];
    let src_gen_funcs: [fn(u64) -> u64; 4];
    let promotion_rank: u64;
    let our_king: u64;

    if pos.white_to_move {
        target_gen_funcs = analysis_tools::w_pawn_target_gen_funcs();
        src_gen_funcs = analysis_tools::w_pawn_src_gen_funcs();
        promotion_rank = RANK_8;
        our_king = pos.w_pieces[d!(Piece::King)]
    } else {
        target_gen_funcs = analysis_tools::b_pawn_target_gen_funcs();
        src_gen_funcs = analysis_tools::b_pawn_src_gen_funcs();
        promotion_rank = RANK_1;
        our_king = pos.b_pieces[d!(Piece::King)]
    }
    
    // Only one the push or capture mask should be applied
    let mask: u64;
    if matches!(move_type, PawnMove::SinglePush | PawnMove::DoublePush) {
        mask = push_mask
    } else {
        mask = capture_mask
    }

    let targets = target_gen_funcs[d!(move_type)](pos) & mask;
    let srcs = src_gen_funcs[d!(move_type)](targets);
    let target_vec = bt::forward_scan(targets);
    let src_vec = bt::forward_scan(srcs);

    assert_eq!(target_vec.len(), src_vec.len());
    for i in 0..target_vec.len() {
        let src = src_vec[i];
        let target = target_vec[i];
        // Check if the pawn is pinned, only allow moves along the axis with
        // our king
        if src & pinned_pieces != EMPTY_BB {
            let pin_mask = bt::ray_axis(
                our_king,
                src
            );
            if target & pin_mask == EMPTY_BB {
                continue;
            }
        }
        // Check if the target is a promotion square
        if target & promotion_rank == EMPTY_BB {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    &Piece::Pawn,
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

fn generate_jumping_moves(
    move_vec: &mut Vec<Move>,
    pos: &Position,
    piece: JumpingPiece,
    our_pieces: &[u64; 7],
    maps: &Maps,
    unsafe_squares: u64,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64,
) {
    let srcs;
    let map;
    let moved_piece;
    match piece {
        JumpingPiece::Knight => {
            srcs = our_pieces[d!(Piece::Knight)];
            map = &maps.knight;
            moved_piece = Piece::Knight;
        },
        JumpingPiece::King => {
            srcs = our_pieces[d!(Piece::King)];
            map = &maps.king;
            moved_piece = Piece::King;
        }
    }
    let src_vec = bt::forward_scan(srcs);
    for src in src_vec {
        let mut targets = map[bt::ilsb(&src)] & !our_pieces[d!(Piece::Any)];
        // Remove unsafe squares i.e. squares attacked by opponent pieces
        // from the available target sqaures for the king
        if matches!(piece, JumpingPiece::King) {
            targets &= !unsafe_squares;
        } else {
            // Only allow moves which either capture a checking piece or blocks
            // the check. These masks should be a FILLED_BB when no check.
            targets &= capture_mask | push_mask;
            if src & pinned_pieces != 0 {
                // If knight is pinned, there are no legal moves
                continue;
            }
        }
        let target_vec = bt::forward_scan(targets);
        for target in target_vec {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    &moved_piece,
                    PromotionPiece::None,
                    SpecialMove::None,
                    pos,
                )
            )
        }
    }
}

fn generate_sliding_moves(
    move_vec: &mut Vec<Move>,
    pos: &Position,
    piece: SlidingPiece,
    f_pieces: &[u64; 7],
    maps: &Maps,
    capture_mask: u64,
    push_mask: u64,
    pinned_pieces: u64,
) {
    let srcs: u64;
    let masks: Vec<&[u64; 64]>;
    let moved_piece;
    match piece {
        SlidingPiece::Bishop => {
            srcs = f_pieces[d!(Piece::Bishop)];
            masks = vec![&maps.diag, &maps.adiag];
            moved_piece = Piece::Bishop;
        },
        SlidingPiece::Rook => {
            srcs = f_pieces[d!(Piece::Rook)];
            masks = vec![&maps.file, &maps.rank];
            moved_piece = Piece::Rook;
        },
        SlidingPiece::Queen => {
            srcs = f_pieces[d!(Piece::Queen)];
            masks = vec![&maps.diag, &maps.adiag, &maps.file, &maps.rank];
            moved_piece = Piece::Queen;
        }
    }
    let src_vec = bt::forward_scan(srcs);
    for src in src_vec {
        let mut targets: u64 = EMPTY_BB;
        for mask in &masks {
            targets |= bt::hyp_quint(pos.occ, src, mask);
        }
        targets &= !f_pieces[d!(Piece::Any)];
        targets &= capture_mask | push_mask;
        // If piece is pinned, it can only move the direction directly to 
        // or from the king
        if pinned_pieces & src != EMPTY_BB {
            let pin_mask = bt::ray_axis(
                f_pieces[d!(Piece::King)], src
            );
            targets &= pin_mask;
        }
        let target_vec = bt::forward_scan(targets);
        for target in target_vec {
            move_vec.push(
                Move::new(
                    target,
                    src,
                    &moved_piece,
                    PromotionPiece::None,
                    SpecialMove::None,
                    pos,
                )
            )
        }
    }
}

// Special Moves

fn generate_promotions(
    move_vec: &mut Vec<Move>, 
    pos: &Position, 
    target: u64, 
    src: u64
) {
    for piece in PromotionPiece::iterator() {
        move_vec.push(
            Move::new(
                target,
                src,
                &Piece::Pawn,
                piece,
                SpecialMove::Promotion,
                pos,                    
            )
        )
    }
}

fn generate_en_passant_moves(
    move_vec: &mut Vec<Move>,
    pos: &Position,
    capture_mask: u64,
    push_mask: u64,
    maps: &Maps
) {
    let target = pos.en_passant_target_sq & push_mask;
    let srcs;    
    let captured_pawn: u64;
    let capture_rank: u64;
    let our_pieces;
    let their_pieces;
    if pos.white_to_move {
        srcs = analysis_tools::w_pawn_en_passant(pos);
        captured_pawn = bt::south_one(target);
        capture_rank = RANK_4;
        our_pieces = &pos.w_pieces;
        their_pieces = &pos.b_pieces;
    } else {
        srcs = analysis_tools::b_pawn_en_passant(pos);
        captured_pawn = bt::north_one(target);
        capture_rank = RANK_5;
        our_pieces = &pos.b_pieces;
        their_pieces = &pos.w_pieces;
    }
    let src_vec = bt::forward_scan(srcs);
    for src in src_vec {
        if captured_pawn & capture_mask != EMPTY_BB {
            // Check rare en passant case that may occur if the king is on the
            // same rank as the pawns involved in the en passant capture where
            // an en passant capture may reveal a discovered check
            if our_pieces[d!(Piece::King)] & capture_rank != EMPTY_BB {
                let occ = pos.occ ^ (src | captured_pawn);
                let king_file_attacks = bt::hyp_quint(
                    occ, our_pieces[d!(Piece::King)], &maps.file
                );
                if king_file_attacks 
                    & (their_pieces[d!(Piece::Rook)] 
                        | their_pieces[d!(Piece::Queen)]
                    ) != EMPTY_BB {
                    continue;
                }
            }
            move_vec.push(
                Move::new(
                    target,
                    src,
                    &Piece::Pawn,
                    PromotionPiece::None,
                    SpecialMove::EnPassant,
                    pos,
                )
            )
        }
    }
}

fn generate_castling_moves(
    move_vec: &mut Vec<Move>,
    pos: &Position,
    castling_masks: &[u64; 4],
    castling_rights: &[bool; 2],
    f_pieces: &[u64; 7], 
    unsafe_squares: u64
) {
    let src = f_pieces[d!(Piece::King)];
    for i in 0..2 {
        if castling_rights[i] && castling_masks[i*2] & pos.occ & unsafe_squares == 0 {
            move_vec.push(
                Move::new(
                    castling_masks[i*2+1],
                    src,
                    &Piece::King,
                    PromotionPiece::None,
                    SpecialMove::Castling,
                    pos,
                )
            )
        }
    }
}

pub fn generate_moves(pos: &Position, maps: &Maps) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let our_pieces: &[u64; 7];
    let castle_masks: &[u64; 4];
    let castle_rights;
    let color;
    if pos.white_to_move {
        our_pieces = &pos.w_pieces;
        castle_masks = &W_CASTLE;
        castle_rights = [pos.w_kingside_castle, pos.w_queenside_castle];
        color = Color::White;
    } else {
        our_pieces = &pos.b_pieces;
        castle_masks = &B_CASTLE;
        castle_rights = [pos.b_kingside_castle, pos.b_kingside_castle];
        color = Color::Black;
    }
    let (unsafe_squares, attackers) = pos.get_unsafe_squares_for(&color, maps);
    let pinned_pieces = pos.get_pinned_pieces_for(&color, maps);
    // Number of pieces placing the king in check
    let n_attackers = attackers.count_ones();
    let mut capture_mask: u64 = FILLED_BB;
    let mut push_mask: u64 = FILLED_BB;
    if n_attackers > 1 {
        // If the king is in double check, only king moves to safe sqaures are valid
        generate_jumping_moves(
            &mut moves, pos, JumpingPiece::King, our_pieces, maps,
            unsafe_squares, capture_mask, push_mask, pinned_pieces
        );
        return moves;
    }
    if n_attackers == 1 {
        // This means the king is in single check so moves are only legal if
        // 1. It moves the king out of check
        // 2. The attacking piece is captured
        // 3. The attacking piece is blocked, if the piece is a sliding piece
        capture_mask = attackers;
        if pos.piece_at_is_slider(attackers) {
            // If the attacker is a sliding piece, then check can be blocked by
            // another piece moving to the intervening squares
            push_mask = bt::create_push_mask(attackers, our_pieces[d!(Piece::King)])
        } else {
            // Not a slider so it can only be captured;
            // give no options to block
            push_mask = EMPTY_BB
        }
    }

    for move_type in PawnMove::iter() {
        generate_pawn_moves(
            &mut moves, 
            pos, 
            move_type,
            capture_mask,
            push_mask,
            pinned_pieces
        )
    }

    for piece in JumpingPiece::iter() {
        generate_jumping_moves(
            &mut moves,
            pos,
            piece,
            our_pieces,
            maps,
            unsafe_squares,
            capture_mask,
            push_mask,
            pinned_pieces
        )
    }

    for piece in SlidingPiece::iter() {
        generate_sliding_moves(
            &mut moves,
            pos,
            piece, 
            our_pieces,
            maps,
            capture_mask,
            push_mask,
            pinned_pieces
        )
    }
    // Bishop moves
    generate_sliding_moves(
        &mut moves, pos, SlidingPiece::Bishop, our_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    );
    // Rook moves
    generate_sliding_moves(
        &mut moves, pos, SlidingPiece::Rook, our_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    );
    // Queen moves
    generate_sliding_moves(
        &mut moves, pos, SlidingPiece::Queen, our_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    );
    // Castling only allowed if not in check
    if n_attackers == 0 {
        generate_castling_moves(
            &mut moves, pos, castle_masks, &castle_rights, our_pieces, unsafe_squares
        );
    }
    if pos.en_passant_target_sq & push_mask != EMPTY_BB {
        generate_en_passant_moves( &mut moves, pos, capture_mask, push_mask, maps);
    }
    return moves;
}

pub fn apply_move(mut pos: Position, mv: &Move) -> Position {
    let our_pieces;
    let their_pieces;
    if pos.white_to_move {
        our_pieces = &mut pos.w_pieces;
        their_pieces = &mut pos.b_pieces;
    } else {
        our_pieces = &mut pos.b_pieces;
        their_pieces = &mut pos.w_pieces;
    }
    // Common operations for all moves
    let move_mask = mv.src | mv.target;
    pos.free |= mv.src; // Source squares must be free now
    pos.occ &= !mv.src;
    pos.free &= !mv.target; // Target sqaures must be occupied
    pos.occ |= mv.target;
    // Our bitboards must be flipped at target and source
    our_pieces[d!(Piece::Any)] ^= move_mask; 
    our_pieces[d!(mv.moved_piece)] ^= move_mask;
    // Free the squares on the their bitboards if the piece is a capture
    if mv.is_capture {
        their_pieces[d!(mv.captured_piece)] ^= mv.target;
        their_pieces[d!(Piece::Any)] ^= mv.target;
        if matches!(mv.captured_piece, Piece::Rook) && mv.target & ROOK_START != 0 {
            // If a rook on its starting square is captured, always set the
            // castling rights as false.
            match mv.target {
                WQROOK => pos.w_queenside_castle = false,
                WKROOK => pos.w_kingside_castle = false,
                BQROOK => pos.b_queenside_castle = false,
                BKROOK => pos.b_kingside_castle = false,
                _ => ()
            }
        }
    }
    // Similarly, if a rook has been moved from its starting square, always
    // set the castling rights as false
    if matches!(mv.moved_piece, Piece::Rook) && mv.src & ROOK_START != 0 {
        match mv.src {
            WQROOK => pos.w_queenside_castle = false,
            WKROOK => pos.w_kingside_castle = false,
            BQROOK => pos.b_queenside_castle = false,
            BKROOK => pos.b_kingside_castle = false,
            _ => ()
        }
    }
    // Set en passant target sq to empty, this will be set to the relevant
    // value for dbl pawn pushes later
    pos.en_passant_target_sq = EMPTY_BB;
    // Reset the halfmove clock if a pawn is moved or a capture has taken
    // place. Else, increment the halfmove clock
    if mv.is_capture || matches!(mv.moved_piece, Piece::Pawn) {
        pos.halfmove_clock = 0;
    } else {
        pos.halfmove_clock += 1;
    }
    // Increment the fullmove clock if black has moved
    if !pos.white_to_move {
        pos.fullmove_clock += 1;
    }
    match mv.special_move_flag {
        SpecialMove::None => (),
        SpecialMove::Promotion => {
            // Set target square on promotion piece bitboard
            our_pieces[d!(mv.promotion_piece)] |= mv.target;
            // Unset the pawn from our pawn bitboard
            our_pieces[d!(Piece::Pawn)] ^= mv.target;
        },
        SpecialMove::Castling => {
            assert!(matches!(mv.moved_piece, Piece::King));
            // For castling moves, we also need the update the rook
            // bitboard and the our universal bitboard
            // Calculate if kingside or queenside castle
            let rook_castle_mask: u64;
            if mv.target.trailing_zeros() % 8 == 6 {
                // For kingside castle, the rook has transported from a
                // position one east of the target square to one west
                rook_castle_mask = mv.target << 1 | mv.target >> 1;
            } else {
                // For the queenside castle, the rook has transported from
                // a position 2 squares west of the target square to the
                // position 1 east of the target sqaure
                assert!(mv.target.trailing_zeros() % 8 == 2);
                rook_castle_mask = mv.target << 1 | mv.target >> 2;
            }
            our_pieces[d!(Piece::Rook)] ^= rook_castle_mask;
            our_pieces[d!(Piece::Any)] ^= rook_castle_mask;
            // Disallow any more castling moves if a castle has occurred
            if pos.white_to_move {
                pos.w_kingside_castle = false;
                pos.w_queenside_castle = false;
            } else {
                pos.b_kingside_castle = false;
                pos.b_queenside_castle = false;
            }
        },
        SpecialMove::EnPassant => {
            assert!(pos.en_passant_target_sq != 0);
            let ep_capture_sq;
            if pos.white_to_move {
                // If white made the en passant capture, then the square at
                // which the capture takes place is on square south of the
                // target square
                ep_capture_sq = bt::south_one(mv.target)
            } else {
                // Opposite for black
                ep_capture_sq = bt::north_one(mv.target)
            }
            // Reflect the capture on the opponent bitboards
            their_pieces[d!(Piece::Any)] ^= ep_capture_sq;
            their_pieces[d!(Piece::Pawn)] ^= ep_capture_sq;
        },
        SpecialMove::DoublePush => {
            // Set enpassant square if the move was a double push
            if pos.white_to_move {
                // If white made the double pawn push, then the ep target
                // square must be one square north of the source
                pos.en_passant_target_sq = bt::north_one(mv.src)
            } else {
                // Vice versa for black
                pos.en_passant_target_sq = bt::south_one(mv.src)
            }
        }
    }
    // Change the turn
    pos.white_to_move = !pos.white_to_move;
    return pos
}

#[cfg(test)]
mod tests;