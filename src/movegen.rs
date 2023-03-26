use super::*;
use movelist::MoveList;
use position::Position;
use types::{Axis, GenType, PieceType};

/// Generate all legal moves in a position
pub fn generate_all<T: MoveList>(pos: &Position, movelist: &mut T) {
    let checkers = pos.checkers();

    if checkers.pop_count() == 0 {
        generate(GenType::NonEvasions, pos, movelist);
    } else {
        generate(GenType::Evasions(checkers), pos, movelist)
    }
}

pub fn generate<T: MoveList>(gt: GenType, pos: &Position, movelist: &mut T) {
    let us = pos.us();

    let targets = match gt {
        GenType::NonEvasions => {
            // Castling is only allowed when not in check
            generate_castles(pos, movelist);
            !us.all
        }
        GenType::Evasions(checker) => {
            debug_assert_ne!(checker.pop_count(), 0);
            if checker.pop_count() > 1 {
                // Only king moves are legal in double check
                generate_king_moves(pos, movelist, gt);
                return;
            }
            !us.all & us.king.between_bb(checker)
        }
        GenType::Captures => pos.them().all,
    };

    let pinned = pos.pinned();

    generate_moves(PieceType::Rook, pos, movelist, pinned, targets);
    generate_moves(PieceType::Knight, pos, movelist, pinned, targets);
    generate_moves(PieceType::Bishop, pos, movelist, pinned, targets);
    generate_moves(PieceType::Queen, pos, movelist, pinned, targets);

    generate_pawn_moves(pos, movelist, pinned, targets);
    generate_king_moves(pos, movelist, gt);
}

fn generate_king_moves<T: MoveList>(pos: &Position, movelist: &mut T, gt: GenType) {
    let us = pos.us();
    let from = us.king;
    let mut targets = from.king_lu() & !us.all & !pos.unsafe_sq();

    if let GenType::Captures = gt {
        targets &= pos.them().all;
    }

    let quiet = targets & pos.free;
    // Add quiet moves
    for to in targets & quiet {
        movelist.add_quiet(from, to);
    }
    // Add captures
    for to in targets ^ quiet {
        movelist.add_capture(from, to);
    }
}

fn generate_pawn_moves<T: MoveList>(pos: &Position, movelist: &mut T, pinned: BB, targets: BB) {
    let us = pos.us();
    let them = pos.them();

    // Filter pawns
    let pinned = us.pawn & pinned;
    // Pawns pinned along a rank cannot move
    let pawns = us.pawn ^ (pinned & us.king.rank());

    let push_only = pinned & us.king.file();
    let lcap_only;
    let rcap_only;

    match pos.stm {
        position::Color::White => {
            lcap_only = pinned & us.king.adiag();
            rcap_only = pinned & us.king.diag();
        }
        position::Color::Black => {
            lcap_only = pinned & us.king.diag();
            rcap_only = pinned & us.king.adiag();
        }
    };

    let capt_only = lcap_only | rcap_only;

    let pawns_on_7 = pawns & pos.rank_7();
    let pawns_not_on_7 = pawns ^ pawns_on_7;

    // Single and double pushes
    let mut bb_1 = pos.push(pawns_not_on_7 & !capt_only) & pos.free;
    let mut bb_2 = pos.push(bb_1 & pos.rank_3()) & pos.free;

    bb_1 &= targets;
    bb_2 &= targets;

    for (from, to) in std::iter::zip(pos.push_back(bb_1), bb_1) {
        movelist.add_quiet(from, to);
    }
    for (from, to) in std::iter::zip(pos.push_back_two(bb_2), bb_2) {
        movelist.add_double_pawn_push(from, to);
    }

    // Promotions
    let bb_1 = pos.push(pawns_on_7 & !capt_only) & pos.free & targets;
    let bb_2 = pos.lcap(pawns_on_7 & !(push_only | rcap_only)) & pos.occ & targets;
    let bb_3 = pos.rcap(pawns_on_7 & !(push_only | lcap_only)) & pos.occ & targets;

    for (from, to) in std::iter::zip(pos.push_back(bb_1), bb_1) {
        movelist.add_promotions(from, to);
    }
    for (from, to) in std::iter::zip(pos.lcap_back(bb_2), bb_2) {
        movelist.add_promo_captures(from, to)
    }
    for (from, to) in std::iter::zip(pos.rcap_back(bb_3), bb_3) {
        movelist.add_promo_captures(from, to)
    }

    // Captures
    let bb_1 = pos.lcap(pawns_not_on_7 & !(push_only | rcap_only)) & pos.occ & targets;
    let bb_2 = pos.rcap(pawns_not_on_7 & !(push_only | lcap_only)) & pos.occ & targets;

    for (from, to) in std::iter::zip(pos.lcap_back(bb_1), bb_1) {
        movelist.add_capture(from, to)
    }
    for (from, to) in std::iter::zip(pos.rcap_back(bb_2), bb_2) {
        movelist.add_capture(from, to)
    }

    // Enpassant
    if pos.ep_sq.is_empty() {
        return;
    };

    let ep_captured_pawn = pos.push_back(pos.ep_sq);

    if ((ep_captured_pawn | pos.ep_sq) & targets).is_empty() {
        return;
    };

    let s_1 = pos.lcap_back(pos.ep_sq) & (pawns ^ push_only ^ rcap_only);
    let s_2 = pos.rcap_back(pos.ep_sq) & (pawns ^ push_only ^ lcap_only);

    for from in s_1 | s_2 {
        if (us.king & pos.rank_5()).is_not_empty() {
            let occ = pos.occ & !(from | ep_captured_pawn);
            if (us.king.hyp_quint(occ, Axis::Rank) & (them.rook | them.queen)).is_not_empty() {
                continue;
            }
        }
        movelist.add_ep(from, pos.ep_sq)
    }
}

fn generate_moves<T: MoveList>(
    pt: PieceType,
    pos: &Position,
    movelist: &mut T,
    pinned: BB,
    targets: BB,
) {
    let us = pos.us();
    let bb = us[pt];

    let attack_gen = match pt {
        PieceType::Knight => BB::knight_lu_,
        PieceType::Bishop => BB::bishop_lu,
        PieceType::Rook => BB::rook_lu,
        PieceType::Queen => BB::queen_lu,
        _ => panic!("invalid PieceType for attacks"),
    };

    for from in bb {
        let mut targets = attack_gen(&from, pos.occ) & targets;
        // Pinned pieces, allow only moves towards or away from king
        if (from & pinned).is_not_empty() {
            targets &= us.king.through_bb(from)
        }
        // Add quiet moves
        let quiet = targets & pos.free;
        for to in quiet {
            movelist.add_quiet(from, to)
        }
        // Add captures
        for to in targets ^ quiet {
            movelist.add_capture(from, to)
        }
    }
}

fn generate_castles<T: MoveList>(pos: &Position, movelist: &mut T) {
    let from = pos.us().king;
    let unsafe_squares = pos.unsafe_sq();

    if pos.can_ksc()
        && (pos.kingside_castle_mask() & pos.occ).is_empty()
        && (pos.kingside_castle_mask() & unsafe_squares).is_empty()
    {
        movelist.add_short_castle(from, from.east_two());
    }
    // Queenside castle
    if pos.can_qsc()
        && (pos.queenside_castle_free_mask() & pos.occ).is_empty()
        && (pos.queenside_castle_safety_mask() & unsafe_squares).is_empty()
    {
        movelist.add_long_castle(from, from.west_two());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use movelist::UnorderedList;
    use test_case::test_case;

    fn setup(fen: &str) -> (Position, UnorderedList) {
        return (Position::from_fen(fen).unwrap(), UnorderedList::new());
    }

    #[test_case(STARTPOS, 16; "starting")]
    #[test_case(TPOS2, 8; "position_two")]
    fn test_pawn_movegen(fen: &str, expected_nodes: usize) {
        let (pos, mut movelist) = setup(fen);
        generate_all(&pos, &mut movelist);
        let pawnmoves: Vec<_> = movelist
            .iter()
            .filter(|mv| matches!(pos.us().pt_at(mv.from()), Some(PieceType::Pawn)))
            .collect();
        assert_eq!(expected_nodes, pawnmoves.len());
    }

    #[test_case(STARTPOS, 4; "starting")]
    #[test_case(TPOS2, 11; "position_two")]
    fn test_knight_movegen(fen: &str, expected_nodes: usize) {
        let (pos, mut movelist) = setup(fen);
        generate_all(&pos, &mut movelist);
        let knightmoves: Vec<_> = movelist
            .iter()
            .filter(|mv| matches!(pos.us().pt_at(mv.from()), Some(PieceType::Knight)))
            .collect();
        assert_eq!(expected_nodes, knightmoves.len());
    }

    #[test_case(STARTPOS, 0; "starting")]
    #[test_case(TPOS2, 2; "position_two")]
    fn test_king_movegen(fen: &str, expected_nodes: usize) {
        let (pos, mut movelist) = setup(fen);
        generate_king_moves(&pos, &mut movelist, GenType::NonEvasions);
        assert_eq!(expected_nodes, movelist.len());
    }

    #[test_case(STARTPOS, 0; "starting")]
    #[test_case(TPOS2, 2; "position_two")]
    fn test_castling_movegen(fen: &str, expected_nodes: usize) {
        let (pos, mut movelist) = setup(fen);
        generate_castles(&pos, &mut movelist);
        assert_eq!(expected_nodes, movelist.len());
    }

    #[test_case(STARTPOS, 20, 0; "starting")]
    #[test_case(TPOS2, 48, 8; "position_two")]
    #[test_case(TPOS3, 14, 1; "position_three")]
    #[test_case(TPOS4, 6, 0; "position_four")]
    fn test_movegen(fen: &str, expected_nodes: i32, expected_captures: usize) {
        let (pos, mut movelist) = setup(fen);
        generate_all(&pos, &mut movelist);
        let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
        assert_eq!(expected_nodes, movelist.len() as i32, "nodes");
        assert_eq!(expected_captures, captures.len(), "captures")
    }

    #[test_case(TPOS2, 8; "position_two")]
    #[test_case(TPOS3, 1; "position_three")]
    fn test_captures_movegen(fen: &str, expected_captures: usize) {
        let (pos, mut movelist) = setup(fen);
        generate(GenType::Captures, &pos, &mut movelist);
        assert_eq!(movelist.len(), expected_captures);
        let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
        assert_eq!(captures.len(), expected_captures);
    }

    #[test]
    fn test_evasion_movegen() {
        let (pos, mut movelist) = setup("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1");
        generate(GenType::Evasions(pos.checkers()), &pos, &mut movelist);
        assert_eq!(movelist.len(), 11)
    }
}
