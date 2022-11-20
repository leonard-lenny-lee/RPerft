use crate::common::*;
use crate::position::Position;
use crate::global::maps::Maps;

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
    pub fn new(target_sq: u64, src_sq: u64, moved_piece: &Piece, 
        promotion_piece: PromotionPiece, special_move_flag: SpecialMove, 
        position: &Position) -> Move {
            let o_pieces;
            if position.white_to_move {
                o_pieces = &position.b_pieces;
            } else {
                o_pieces = &position.w_pieces;
            }
            let is_capture = o_pieces[0] & target_sq != 0;
            let mut captured_piece = Piece::Any;
            if is_capture {
                for piece in Piece::iter_pieces() {
                    if o_pieces[piece as usize] & target_sq != 0 {
                        // Identified which piece has been captured
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
    position: &Position, move_type: PawnMove, capture_mask: u64, push_mask: u64,
    pinned_pieces: u64
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let targets: u64;
    let srcs: u64;
    let promotion_rank: u64;
    let king_bb: u64;
    if position.white_to_move {
        match move_type {
            PawnMove::SinglePush => {
                targets = position.get_wpawn_sgl_pushes() & push_mask;
                srcs = targets >> 8;
            },
            PawnMove::DoublePush => {
                targets = position.get_wpawn_dbl_pushes() & push_mask;
                srcs = targets >> 16;
            },
            PawnMove::CaptureLeft => {
                targets = position.get_wpawn_left_captures() & capture_mask;
                srcs = targets >> 7;
            },
            PawnMove::CaptureRight => {
                targets = position.get_wpawn_right_captures() & capture_mask;
                srcs = targets >> 9;
            }
        }
        promotion_rank = RANK_8;
        king_bb = position.w_pieces[Piece::King as usize];
    } else {
        match move_type {
            PawnMove::SinglePush => {
                targets = position.get_bpawn_sgl_pushes() & push_mask;
                srcs = targets << 8;
            },
            PawnMove::DoublePush => {
                targets = position.get_bpawn_dbl_pushes() & push_mask;
                srcs = targets << 16;
            },
            PawnMove::CaptureLeft => {
                targets = position.get_bpawn_left_captures() & capture_mask;
                srcs = targets << 9;
            }
            PawnMove::CaptureRight => {
                targets = position.get_bpawn_right_captures() & capture_mask;
                srcs = targets << 7;
            }
        }
        promotion_rank = RANK_1;
        king_bb = position.b_pieces[Piece::King as usize];
    }
    let target_vec = bittools::forward_scan(targets);
    let src_vec = bittools::forward_scan(srcs);
    for i in 0..target_vec.len() {
        let src = src_vec[i];
        let target = target_vec[i];
        if src & pinned_pieces != 0 {
            // If pawn in pinned, it can only along the pin mask from the king
            let pin_mask = bittools::ray_axis(king_bb, src);
            if target & pin_mask == 0 {
                continue;
            }
        }
        if target & promotion_rank == 0 {
            moves.push(
                Move::new(
                    target,
                    src, 
                    &Piece::Pawn,
                    PromotionPiece::None, 
                    SpecialMove::None,
                    position)
            )
        } else {
            let mut promotions = generate_promotions(position, target, src);
            moves.append(&mut promotions);
        }
    }
    return moves;
}

fn generate_jumping_moves(
    position: &Position, piece: JumpingPiece, f_pieces: &[u64; 7], maps: &Maps,
    unsafe_squares: u64, capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let srcs;
    let map;
    let moved_piece;
    match piece {
        JumpingPiece::Knight => {
            srcs = f_pieces[Piece::Knight as usize];
            map = &maps.knight;
            moved_piece = Piece::Knight;
        },
        JumpingPiece::King => {
            srcs = f_pieces[Piece::King as usize];
            map = &maps.king;
            moved_piece = Piece::King;
        }
    }
    let src_vec = bittools::forward_scan(srcs);
    for src in src_vec {
        let mut targets = map[bittools::ilsb(&src)] & !f_pieces[0];
        if matches!(piece, JumpingPiece::King) {
            // Remove unsafe squares i.e. squares attacked by opponent pieces
            // from the available target sqaures for the king
            targets &= !unsafe_squares;
        } else {
            targets &= capture_mask | push_mask;
            if src & pinned_pieces != 0 {
                // If knight is pinned, there are no legal moves
                continue;
            }
        }
        let target_vec = bittools::forward_scan(targets);
        for target in target_vec {
            moves.push(
                Move::new(
                    target,
                    src,
                    &moved_piece,
                    PromotionPiece::None,
                    SpecialMove::None,
                    position,
                )
            )
        }
    }
    return moves;
}

fn generate_sliding_moves(
    position: &Position, piece: SlidingPiece, f_pieces: &[u64; 7], maps: &Maps,
    capture_mask: u64, push_mask: u64, pinned_pieces: u64,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let srcs: u64;
    let masks: Vec<&[u64; 64]>;
    let moved_piece;
    match piece {
        SlidingPiece::Bishop => {
            srcs = f_pieces[Piece::Bishop as usize];
            masks = vec![&maps.diag, &maps.adiag];
            moved_piece = Piece::Bishop;
        },
        SlidingPiece::Rook => {
            srcs = f_pieces[Piece::Rook as usize];
            masks = vec![&maps.file, &maps.rank];
            moved_piece = Piece::Rook;
        },
        SlidingPiece::Queen => {
            srcs = f_pieces[Piece::Queen as usize];
            masks = vec![&maps.diag, &maps.adiag, &maps.file, &maps.rank];
            moved_piece = Piece::Queen;
        }
    }
    let src_vec = bittools::forward_scan(srcs);
    for src in src_vec {
        let mut targets: u64 = 0;
        for mask in &masks {
            targets |= bittools::hyp_quint(position.occ, src, mask);
        }
        targets &= !f_pieces[Piece::Any as usize];
        targets &= capture_mask | push_mask;
        if pinned_pieces & src != 0 {
            // If piece is pinned, it can only move the direction directly to /
            // from the king
            let pin_mask = bittools::ray_axis(f_pieces[Piece::King as usize], src);
            targets &= pin_mask;
        }
        let target_vec = bittools::forward_scan(targets);
        for target in target_vec {
            moves.push(
                Move::new(
                    target,
                    src,
                    &moved_piece,
                    PromotionPiece::None,
                    SpecialMove::None,
                    position,
                )
            )
        }
    }
    return moves;
}

// Special Moves

fn generate_promotions(
    position: &Position, target: u64, src: u64
) -> Vec<Move> {
    let mut promotions: Vec<Move> = Vec::new();
    for piece in PromotionPiece::iterator() {
        promotions.push(
            Move::new(
                target,
                src,
                &Piece::Pawn,
                piece,
                SpecialMove::Promotion,
                position,                    
            )
        )
    }
    return promotions;
}

fn generate_en_passant_moves(
    position: &Position, capture_mask: u64, push_mask: u64, maps: &Maps
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let target_sq = position.en_passant_target_sq & push_mask;
    if target_sq == 0 {
        return moves;
    }
    let mut src_vec: Vec<u64> = Vec::new();
    let target_cap: u64;
    let cap_rank: u64;
    let f_pieces;
    let o_pieces;
    if position.white_to_move {
        src_vec.push(position.get_wpawn_left_en_passant() >> 7);
        src_vec.push(position.get_wpawn_right_en_passant() >> 9);
        target_cap = target_sq >> 8;
        cap_rank = RANK_4;
        f_pieces = &position.w_pieces;
        o_pieces = &position.b_pieces;
    } else {
        src_vec.push(position.get_bpawn_left_en_passant() << 9);
        src_vec.push(position.get_bpawn_right_en_passant() << 7);
        target_cap = target_sq << 8;
        cap_rank = RANK_5;
        f_pieces = &position.b_pieces;
        o_pieces = &position.w_pieces;
    }
    for src in src_vec {
        if src != 0 && target_cap & capture_mask != 0 {
            // Check rare en passant case that may occur if the king is on the
            // same rank as the pawns involved in the en passant capture
            if f_pieces[Piece::King as usize] & cap_rank != 0 {
                let occ = position.occ ^ (src | target_cap);
                let king_file_attacks = bittools::hyp_quint(
                    occ, f_pieces[Piece::King as usize], &maps.file);
                if king_file_attacks 
                    & (o_pieces[Piece::Rook as usize] 
                        | o_pieces[Piece::Queen as usize]) != 0 {
                    continue;
                }
            }
            moves.push(
                Move::new(
                    target_sq,
                    src,
                    &Piece::Pawn,
                    PromotionPiece::None,
                    SpecialMove::EnPassant,
                    position,
                )
            )
        }
    }
    return moves;
}

fn generate_castling_moves(
    position: &Position, castling_masks: &[u64; 4], castling_rights: &[bool; 2], f_pieces: &[u64; 7], 
    unsafe_squares: u64
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let src = f_pieces[Piece::King as usize];
    for i in 0..2 {
        if castling_rights[i] && castling_masks[i*2] & position.occ & unsafe_squares == 0 {
            moves.push(
                Move::new(
                    castling_masks[i*2+1],
                    src,
                    &Piece::King,
                    PromotionPiece::None,
                    SpecialMove::Castling,
                    position,
                )
            )
        }
    }
    return moves;
}

pub fn generate_moves(pos: &Position, maps: &Maps) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let f_pieces: &[u64; 7];
    let castle_masks: &[u64; 4];
    let castle_rights;
    let color;
    if pos.white_to_move {
        f_pieces = &pos.w_pieces;
        castle_masks = &W_CASTLE;
        castle_rights = [pos.w_kingside_castle, pos.w_queenside_castle];
        color = Color::White;
    } else {
        f_pieces = &pos.b_pieces;
        castle_masks = &B_CASTLE;
        castle_rights = [pos.b_kingside_castle, pos.b_kingside_castle];
        color = Color::Black;
    }
    let (unsafe_squares, attackers) = pos.get_unsafe_squares_for(&color, maps);
    let pinned_pieces = pos.get_pinned_pieces_for(&color, maps);
    // Number of pieces placing the king in check
    let n_attackers = attackers.count_ones();
    let mut capture_mask: u64 = 0xffffffffffffffff;
    let mut push_mask: u64 = 0xffffffffffffffff;
    if n_attackers > 1 {
        // If the king is in double check, only king moves to safe sqaures are valid
        moves.append(&mut generate_jumping_moves(
            pos, JumpingPiece::King, f_pieces, maps,
            unsafe_squares, capture_mask, push_mask, pinned_pieces
        ));
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
            push_mask = bittools::create_push_mask(attackers, f_pieces[Piece::King as usize])
        } else {
            // Not a slider so it can only be captured;
            // give no options to block
            push_mask = 0
        }
    }
    // Pawn single pushes
    moves.append(&mut generate_pawn_moves(
        pos, PawnMove::SinglePush, capture_mask, push_mask,
        pinned_pieces,
    ));
    // Pawn double pushes
    moves.append(&mut generate_pawn_moves(
        pos, PawnMove::DoublePush, capture_mask, push_mask, 
        pinned_pieces
    ));
    // Pawn left captures
    moves.append(&mut generate_pawn_moves(
        pos, PawnMove::CaptureLeft, capture_mask, 
        push_mask, pinned_pieces
    ));
    // Pawn right captures
    moves.append(&mut generate_pawn_moves(
        pos, PawnMove::CaptureRight, capture_mask, 
        push_mask, pinned_pieces
    ));
    // Knight moves
    moves.append(&mut generate_jumping_moves(
        pos, JumpingPiece::Knight, f_pieces, maps, unsafe_squares,
        capture_mask, push_mask, pinned_pieces
    ));
    // King moves
    moves.append(&mut generate_jumping_moves(
        pos, JumpingPiece::King, f_pieces, maps, unsafe_squares,
        capture_mask, push_mask, pinned_pieces
    ));
    // Bishop moves
    moves.append(&mut generate_sliding_moves(
        pos, SlidingPiece::Bishop, f_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    ));
    // Rook moves
    moves.append(&mut generate_sliding_moves(
        pos, SlidingPiece::Rook, f_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    ));
    // Queen moves
    moves.append(&mut generate_sliding_moves(
        pos, SlidingPiece::Queen, f_pieces, maps, capture_mask,
        push_mask, pinned_pieces
    ));
    // Castling only allowed if not in check
    if n_attackers == 0 {
        moves.append(&mut generate_castling_moves(
            pos, castle_masks, &castle_rights, f_pieces, unsafe_squares
        ));
    }

    if pos.en_passant_target_sq != 0 {
        moves.append(&mut generate_en_passant_moves(pos, capture_mask,
        push_mask, maps));
    }

    return moves;
}

pub fn make_move(mut pos: Position, mv: &Move) -> Position {
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
    our_pieces[Piece::Any as usize] ^= move_mask; 
    our_pieces[mv.moved_piece as usize] ^= move_mask;
    // Free the squares on the their bitboards if the piece is a capture
    if mv.is_capture {
        their_pieces[mv.captured_piece as usize] ^= mv.target;
        their_pieces[Piece::Any as usize] ^= mv.target;
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
            our_pieces[mv.promotion_piece as usize] |= mv.target;
            // Unset the pawn from our pawn bitboard
            our_pieces[Piece::Pawn as usize] ^= mv.target;
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
            our_pieces[Piece::Rook as usize] ^= rook_castle_mask;
            our_pieces[Piece::Any as usize] ^= rook_castle_mask;
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
                ep_capture_sq = bittools::south_one(mv.target)
            } else {
                // Opposite for black
                ep_capture_sq = bittools::north_one(mv.target)
            }
            // Reflect the capture on the opponent bitboards
            their_pieces[Piece::Any as usize] ^= ep_capture_sq;
            their_pieces[Piece::Pawn as usize] ^= ep_capture_sq;
        },
        SpecialMove::DoublePush => {
            // Set enpassant square if the move was a double push
            if pos.white_to_move {
                // If white made the double pawn push, then the ep target
                // square must be one square north of the source
                pos.en_passant_target_sq = bittools::north_one(mv.src)
            } else {
                // Vice versa for black
                pos.en_passant_target_sq = bittools::south_one(mv.src)
            }
        }
    }
    // Change the turn
    pos.white_to_move = !pos.white_to_move;
    return pos
}

#[cfg(test)]
mod tests;