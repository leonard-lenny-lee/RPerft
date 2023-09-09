use super::*;
use movelist::MoveList;
use position::Position;
use types::{Axis, GeneratorType, MoveType, Piece};

#[cfg(test)]
mod tests;

/// Generate all legal moves in a position
pub fn generate_all(position: &Position) -> MoveList {
    let mut movelist = MoveList::new();
    let checkers = position.opponent_checkers();
    if checkers.pop_count() == 0 {
        generate(GeneratorType::NonEvasions, position, &mut movelist);
    } else {
        generate(GeneratorType::Evasions(checkers), position, &mut movelist)
    }
    movelist
}

pub fn generate(gt: GeneratorType, position: &Position, movelist: &mut MoveList) {
    let targets = match gt {
        GeneratorType::NonEvasions => {
            // Castling is only allowed when not in check
            generate_castles(position, movelist);
            !position.us.all
        }
        GeneratorType::Evasions(checker) => {
            if checker.pop_count() > 1 {
                // Only king moves are legal in double check
                generate_king_moves(position, movelist, gt);
                return;
            }
            !position.us.all & position.us.king.between_bb(checker)
        }
        GeneratorType::Captures => position.them.all,
    };

    let pinned = position.our_pinned_pieces();

    generate_moves(Piece::Rook, position, movelist, pinned, targets);
    generate_moves(Piece::Knight, position, movelist, pinned, targets);
    generate_moves(Piece::Bishop, position, movelist, pinned, targets);
    generate_moves(Piece::Queen, position, movelist, pinned, targets);

    generate_pawn_moves(position, movelist, pinned, targets);
    generate_king_moves(position, movelist, gt);
}

#[inline(always)]
fn generate_king_moves(
    position: &Position,
    movelist: &mut MoveList,
    generator_type: GeneratorType,
) {
    let from = position.us.king;
    let mut targets =
        from.lookup_king_attacks() & !position.us.all & !position.opponent_attack_squares();

    if let GeneratorType::Captures = generator_type {
        targets &= position.them.all;
    }

    let quiet_targets = targets & position.free;
    // Add quiet moves
    for to in targets & quiet_targets {
        movelist.add_quiet(from, to);
    }
    // Add captures
    for to in targets ^ quiet_targets {
        movelist.add_capture(from, to);
    }
}

#[inline(always)]
fn generate_pawn_moves(
    position: &Position,
    movelist: &mut MoveList,
    pinned: BitBoard,
    targets: BitBoard,
) {
    // Filter pawns according to if they are pinned and the pin direction
    let pinned = position.us.pawn & pinned;

    // Pawns pinned along a rank cannot move
    let pawns = position.us.pawn ^ (pinned & position.us.king.lookup_rank_mask());

    let push_only = pinned & position.us.king.lookup_file_mask();
    let left_capture_only = pinned & position.left_capture_axis(position.us.king);
    let right_capture_only = pinned & position.right_capture_axis(position.us.king);

    let no_push = left_capture_only | right_capture_only;
    let no_left_capture = right_capture_only | push_only;
    let no_right_capture = left_capture_only | push_only;

    let pawns_on_7th_rank = pawns & position.rank_7();
    let pawns_not_on_7th_rank = pawns ^ pawns_on_7th_rank;

    // Single and double pushes
    let mut bb_1 = position.forward_one(pawns_not_on_7th_rank & !no_push) & position.free;
    let mut bb_2 = position.forward_one(bb_1 & position.rank_3()) & position.free;

    bb_1 &= targets;
    bb_2 &= targets;

    for (from, to) in std::iter::zip(position.back_one(bb_1), bb_1) {
        movelist.add_quiet(from, to);
    }
    for (from, to) in std::iter::zip(position.back_two(bb_2), bb_2) {
        movelist.add(from, to, MoveType::DoublePawnPush);
    }

    // Promotions
    let bb_1 = position.forward_one(pawns_on_7th_rank & !no_push) & position.free & targets;
    let bb_2 =
        position.capture_left(pawns_on_7th_rank & !no_left_capture) & position.occupied & targets;
    let bb_3 =
        position.capture_right(pawns_on_7th_rank & !no_right_capture) & position.occupied & targets;

    for (from, to) in std::iter::zip(position.back_one(bb_1), bb_1) {
        movelist.add_promotions(from, to);
    }
    for (from, to) in std::iter::zip(position.capture_left_rev(bb_2), bb_2) {
        movelist.add_promotion_captures(from, to)
    }
    for (from, to) in std::iter::zip(position.capture_right_rev(bb_3), bb_3) {
        movelist.add_promotion_captures(from, to)
    }

    // Captures
    let bb_1 = position.capture_left(pawns_not_on_7th_rank & !no_left_capture)
        & position.occupied
        & targets;
    let bb_2 = position.capture_right(pawns_not_on_7th_rank & !no_right_capture)
        & position.occupied
        & targets;

    for (from, to) in std::iter::zip(position.capture_left_rev(bb_1), bb_1) {
        movelist.add_capture(from, to);
    }
    for (from, to) in std::iter::zip(position.capture_right_rev(bb_2), bb_2) {
        movelist.add_capture(from, to);
    }

    // Enpassant
    if position.en_passant.is_empty() {
        return;
    };

    let en_passant_capture_square = position.back_one(position.en_passant);

    if ((en_passant_capture_square | position.en_passant) & targets).is_empty() {
        return;
    };

    let s_1 = position.capture_left_rev(position.en_passant) & (pawns & !no_left_capture);
    let s_2 = position.capture_right_rev(position.en_passant) & (pawns & !no_right_capture);

    for from in s_1 | s_2 {
        // Check rare case where an ep can reveal a discovered check
        if (position.us.king & position.rank_5()).is_not_empty() {
            // Remove ep pawns from occupied squared
            let occupied = position.occupied & !(from | en_passant_capture_square);
            // Check if king now has direct line of sight to a rook or queen
            if (position.us.king.hyp_quint(occupied, Axis::Rank)
                & (position.them.rook | position.them.queen))
                .is_not_empty()
            {
                continue;
            }
        }
        movelist.add_ep(from, position.en_passant);
    }
}

#[inline(always)]
fn generate_moves(
    piece_type: Piece,
    position: &Position,
    movelist: &mut MoveList,
    pinned: BitBoard,
    targets: BitBoard,
) {
    debug_assert!(matches!(
        piece_type,
        Piece::Rook | Piece::Knight | Piece::Bishop | Piece::Queen
    ));

    type AttackGenerator = fn(&BitBoard, BitBoard) -> BitBoard;

    const ATTACK_GENERATORS: [AttackGenerator; 4] = [
        BitBoard::magic_rook_attacks,
        BitBoard::lookup_knight_attacks_,
        BitBoard::magic_bishop_attacks,
        BitBoard::magic_queen_attacks,
    ];

    let attack_generator = ATTACK_GENERATORS[piece_type as usize - 2];

    for from in position.us[piece_type] {
        let mut targets = attack_generator(&from, position.occupied) & targets;
        // Pinned pieces, allow only moves towards or away from king
        if (from & pinned).is_not_empty() {
            targets &= position.us.king.between_mask(from)
        }
        // Add quiet moves
        let quiet = targets & position.free;
        for to in quiet {
            movelist.add_quiet(from, to);
        }
        // Add captures
        for to in targets ^ quiet {
            movelist.add_capture(from, to);
        }
    }
}

#[inline(always)]
fn generate_castles(position: &Position, movelist: &mut MoveList) {
    let from = position.us.king;
    let unsafe_squares = position.opponent_attack_squares();

    if (position.castling_rights & position.kingside_rook_starting_square()).is_not_empty()
        && (position.short_castle_mask() & position.occupied).is_empty()
        && (position.short_castle_mask() & unsafe_squares).is_empty()
    {
        movelist.add_castle(from, from.east_two(), MoveType::ShortCastle);
    }

    if (position.castling_rights & position.queenside_rook_starting_square()).is_not_empty()
        && (position.queenside_castle_free_mask() & position.occupied).is_empty()
        && (position.queenside_castle_safety_mask() & unsafe_squares).is_empty()
    {
        movelist.add_castle(from, from.west_two(), MoveType::LongCastle);
    }
}
