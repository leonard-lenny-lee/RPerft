use super::*;
use movelist::MoveList;
use position::Position;
use types::{Axis, GenType, MoveType, PieceType};

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
    let targets = match gt {
        GenType::NonEvasions => {
            // Castling is only allowed when not in check
            generate_castles(pos, movelist);
            !pos.us.all
        }
        GenType::Evasions(checker) => {
            debug_assert_ne!(checker.pop_count(), 0);
            if checker.pop_count() > 1 {
                // Only king moves are legal in double check
                generate_king_moves(pos, movelist, gt);
                return;
            }
            !pos.us.all & pos.us.king.between_bb(checker)
        }
        GenType::Captures => pos.them.all,
    };

    let pinned = pos.pinned();

    generate_moves(PieceType::Rook, pos, movelist, pinned, targets);
    generate_moves(PieceType::Knight, pos, movelist, pinned, targets);
    generate_moves(PieceType::Bishop, pos, movelist, pinned, targets);
    generate_moves(PieceType::Queen, pos, movelist, pinned, targets);

    generate_pawn_moves(pos, movelist, pinned, targets);
    generate_king_moves(pos, movelist, gt);
}

#[inline(always)]
fn generate_king_moves<T: MoveList>(pos: &Position, movelist: &mut T, gt: GenType) {
    let from = pos.us.king;
    let mut targets = from.king_lu() & !pos.us.all & !pos.unsafe_sq();

    if let GenType::Captures = gt {
        targets &= pos.them.all;
    }

    let quiet = targets & pos.free;
    // Add quiet moves
    for to in targets & quiet {
        movelist.add(from, to, MoveType::Quiet, pos);
    }
    // Add captures
    for to in targets ^ quiet {
        movelist.add(from, to, MoveType::Capture, pos);
    }
}

#[inline(always)]
fn generate_pawn_moves<T: MoveList>(pos: &Position, movelist: &mut T, pinned: BB, targets: BB) {
    // Filter pawns
    let pinned = pos.us.pawn & pinned;
    // Pawns pinned along a rank cannot move
    let pawns = pos.us.pawn ^ (pinned & pos.us.king.rank());

    let push_only = pinned & pos.us.king.file();
    let lcap_only = pinned & pos.lcap_axis(pos.us.king);
    let rcap_only = pinned & pos.rcap_axis(pos.us.king);

    let no_push = lcap_only | rcap_only;
    let no_lcap = rcap_only | push_only;
    let no_rcap = lcap_only | push_only;

    let pawns_on_7 = pawns & pos.rank_7();
    let pawns_not_on_7 = pawns ^ pawns_on_7;

    // Single and double pushes
    let mut bb_1 = pos.push(pawns_not_on_7 & !no_push) & pos.free;
    let mut bb_2 = pos.push(bb_1 & pos.rank_3()) & pos.free;

    bb_1 &= targets;
    bb_2 &= targets;

    for (from, to) in std::iter::zip(pos.push_back(bb_1), bb_1) {
        movelist.add(from, to, MoveType::Quiet, pos);
    }
    for (from, to) in std::iter::zip(pos.push_back_two(bb_2), bb_2) {
        movelist.add(from, to, MoveType::DoublePawnPush, pos);
    }

    // Promotions
    let bb_1 = pos.push(pawns_on_7 & !no_push) & pos.free & targets;
    let bb_2 = pos.lcap(pawns_on_7 & !no_lcap) & pos.occ & targets;
    let bb_3 = pos.rcap(pawns_on_7 & !no_rcap) & pos.occ & targets;

    for (from, to) in std::iter::zip(pos.push_back(bb_1), bb_1) {
        movelist.add_promotions(from, to, pos);
    }
    for (from, to) in std::iter::zip(pos.lcap_back(bb_2), bb_2) {
        movelist.add_promotion_captures(from, to, pos)
    }
    for (from, to) in std::iter::zip(pos.rcap_back(bb_3), bb_3) {
        movelist.add_promotion_captures(from, to, pos)
    }

    // Captures
    let bb_1 = pos.lcap(pawns_not_on_7 & !no_lcap) & pos.occ & targets;
    let bb_2 = pos.rcap(pawns_not_on_7 & !no_rcap) & pos.occ & targets;

    for (from, to) in std::iter::zip(pos.lcap_back(bb_1), bb_1) {
        movelist.add(from, to, MoveType::Capture, pos);
    }
    for (from, to) in std::iter::zip(pos.rcap_back(bb_2), bb_2) {
        movelist.add(from, to, MoveType::Capture, pos);
    }

    // Enpassant
    if pos.ep_sq.is_empty() {
        return;
    };

    let ep_cap_sq = pos.push_back(pos.ep_sq);

    if ((ep_cap_sq | pos.ep_sq) & targets).is_empty() {
        return;
    };

    let s_1 = pos.lcap_back(pos.ep_sq) & (pawns & !no_lcap);
    let s_2 = pos.rcap_back(pos.ep_sq) & (pawns & !no_rcap);

    for from in s_1 | s_2 {
        // Check rare case where an ep can reveal a discovered check
        if (pos.us.king & pos.rank_5()).is_not_empty() {
            let occ = pos.occ & !(from | ep_cap_sq);
            if (pos.us.king.hyp_quint(occ, Axis::Rank) & (pos.them.rook | pos.them.queen))
                .is_not_empty()
            {
                continue;
            }
        }
        movelist.add(from, pos.ep_sq, MoveType::EnPassant, pos)
    }
}

type AttackGenerator = fn(&BB, BB) -> BB;

const ATTACK_GENERATOR: [AttackGenerator; 4] =
    [BB::rook_lu, BB::knight_lu_, BB::bishop_lu, BB::queen_lu];

#[inline(always)]
fn generate_moves<T: MoveList>(
    pt: PieceType,
    pos: &Position,
    movelist: &mut T,
    pinned: BB,
    targets: BB,
) {
    debug_assert!(
        pt as usize >= 2 && pt as usize <= 5,
        "invalid PieceType for attacks"
    );

    let bb = pos.us[pt];

    let attack_generator = ATTACK_GENERATOR[pt as usize - 2];

    for from in bb {
        let mut targets = attack_generator(&from, pos.occ) & targets;
        // Pinned pieces, allow only moves towards or away from king
        if (from & pinned).is_not_empty() {
            targets &= pos.us.king.through_bb(from)
        }
        // Add quiet moves
        let quiet = targets & pos.free;
        for to in quiet {
            movelist.add(from, to, MoveType::Quiet, pos);
        }
        // Add captures
        for to in targets ^ quiet {
            movelist.add(from, to, MoveType::Capture, pos);
        }
    }
}

#[inline(always)]
fn generate_castles<T: MoveList>(pos: &Position, movelist: &mut T) {
    let from = pos.us.king;
    let unsafe_squares = pos.unsafe_sq();

    if (pos.castling_rights & pos.ksr_start()).is_not_empty()
        && (pos.ksc_mask() & pos.occ).is_empty()
        && (pos.ksc_mask() & unsafe_squares).is_empty()
    {
        movelist.add(from, from.east_two(), MoveType::ShortCastle, pos);
    }

    if (pos.castling_rights & pos.qsr_start()).is_not_empty()
        && (pos.qsc_free_mask() & pos.occ).is_empty()
        && (pos.qsc_safe_mask() & unsafe_squares).is_empty()
    {
        movelist.add(from, from.west_two(), MoveType::LongCastle, pos);
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
            .filter(|mv| matches!(pos.us.pt_at(mv.from()), Some(PieceType::Pawn)))
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
            .filter(|mv| matches!(pos.us.pt_at(mv.from()), Some(PieceType::Knight)))
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
