use super::*;

use std::cmp::Ordering;

use movelist::MoveList;
use position::states::*;
use position::Position;
use pt::*;
use types::{Color, MoveT};

/// Generate all legal moves in a position
pub fn generate_all<T: MoveList>(pos: &Position, movelist: &mut T) {
    match pos.stm {
        Color::White => generate_all_inner::<T, White>(pos, movelist),
        Color::Black => generate_all_inner::<T, Black>(pos, movelist),
    }
}

#[inline(always)]
fn generate_all_inner<T: MoveList, U: State>(pos: &Position, movelist: &mut T) {
    let checkers = pos.checkers::<U>();
    let n_checkers = checkers.pop_count();
    let filter;

    match n_checkers.cmp(&1) {
        Ordering::Greater => {
            // In double check, only king moves are valid
            generate_king_moves::<T, U>(pos, movelist);
            return;
        }
        Ordering::Equal => {
            // In single check, the only valid moves are to capture the checker or block
            filter = pos.us.king.between_bb(checkers);
        }
        Ordering::Less => {
            // Not in check so all squares that are not occupied by our pieces are valid targets
            filter = !pos.us.all;
            // Castling is allowed only when not in check
            generate_castles::<T, U>(pos, movelist);
        }
    }

    let pinned = pos.pinned();

    generate_moves::<T, Rook>(pos, movelist, pinned, filter);
    generate_moves::<T, Knight>(pos, movelist, pinned, filter);
    generate_moves::<T, Bishop>(pos, movelist, pinned, filter);
    generate_moves::<T, Queen>(pos, movelist, pinned, filter);

    generate_pawn_moves::<T, U>(pos, movelist, pinned, filter);
    generate_king_moves::<T, U>(pos, movelist);
}

#[inline(always)]
fn generate_king_moves<T: MoveList, U: State>(pos: &Position, movelist: &mut T) {
    let from = pos.us.king;
    let targets = from.king_attacks_lu() & !pos.us.all & !pos.unsafe_sq::<U>();
    let quiet_targets = targets & pos.free;
    movelist.add_quiets(from, targets & quiet_targets);
    movelist.add_captures(from, targets ^ quiet_targets);
}

#[inline(always)]
fn generate_pawn_moves<T: MoveList, U: State>(
    pos: &Position,
    movelist: &mut T,
    pinned: BitBoard,
    filter: BitBoard,
) {
    // Filter pawns according to if they are pinned and the pin direction
    let pinned = pos.us.pawn & pinned;

    // Pawns pinned along a rank cannot move as they can only move forward or diagonally
    let pawns = pos.us.pawn ^ (pinned & pos.us.king.rank_mask_lu());

    // Pawns pinned along a file or diagonal can only move along those axes
    let push_only = pinned & pos.us.king.file_mask_lu();
    let left_only = pinned & U::l_cap_axis(pos.us.king);
    let right_only = pinned & U::r_cap_axis(pos.us.king);

    // Separate pawns according to direction of movement
    let no_push = left_only | right_only;
    let no_left = right_only | push_only;
    let no_right = left_only | push_only;

    // Separate pawns on whether they can promote
    let on_7 = pawns & U::rank_7();
    let not_on_7 = pawns ^ on_7;

    // Add single and double pushes
    let mut bb_1 = U::push_one(not_on_7 & !no_push) & pos.free;
    let mut bb_2 = U::push_one(bb_1 & U::rank_3()) & pos.free;

    bb_1 &= filter;
    bb_2 &= filter;

    movelist.add_pawn_pushes(U::back_one(bb_1), bb_1);
    movelist.add_double_pawn_pushes(U::back_two(bb_2), bb_2);

    // Add promotions
    let bb_1 = U::push_one(on_7 & !no_push) & pos.free & filter;
    let bb_2 = U::l_cap(on_7 & !no_left) & pos.occ & filter;
    let bb_3 = U::r_cap(on_7 & !no_right) & pos.occ & filter;

    movelist.add_promos(U::back_one(bb_1), bb_1);
    movelist.add_promo_captures(U::l_cap_back(bb_2), bb_2);
    movelist.add_promo_captures(U::r_cap_back(bb_3), bb_3);

    // Add captures
    let bb_1 = U::l_cap(not_on_7 & !no_left) & pos.occ & filter;
    let bb_2 = U::r_cap(not_on_7 & !no_right) & pos.occ & filter;

    movelist.add_pawn_captures(U::l_cap_back(bb_1), bb_1);
    movelist.add_pawn_captures(U::r_cap_back(bb_2), bb_2);

    // Enpassant
    if pos.ep_sq.is_empty() {
        return;
    };

    let ep_capture_sq = U::back_one(pos.ep_sq);

    if ((ep_capture_sq | pos.ep_sq) & filter).is_empty() {
        return;
    };

    let s_1 = U::l_cap_back(pos.ep_sq) & (pawns & !no_left);
    let s_2 = U::r_cap_back(pos.ep_sq) & (pawns & !no_right);

    for from in s_1 | s_2 {
        // Check rare case where an ep can reveal a discovered check along the 5th rank
        if (pos.us.king & U::rank_5()).is_empty() {
            movelist.add_ep(from, pos.ep_sq);
            continue;
        }
        // Check if removing the captured pawn and ep pawn from their squares will reveal a check
        let occ = pos.occ & !(from | ep_capture_sq);
        let king_ray = pos.us.king.hq_rank_attacks(occ);
        if (king_ray & (pos.them.rook | pos.them.queen)).is_not_empty() {
            continue;
        }
        movelist.add_ep(from, pos.ep_sq);
    }
}

#[inline(always)]
fn generate_moves<T: MoveList, U: Piece>(
    pos: &Position,
    movelist: &mut T,
    pinned: BitBoard,
    filter: BitBoard,
) {
    for from in pos.us[U::pt()] {
        let mut targets = U::generate_attacks(from, pos.occ) & filter;
        // For pinned pieces, allow only moves towards or away from king
        if (from & pinned).is_not_empty() {
            targets &= pos.us.king.between_mask(from)
        }
        movelist.add_quiets(from, targets & pos.free);
        movelist.add_captures(from, targets & pos.occ);
    }
}

#[inline(always)]
fn generate_castles<T: MoveList, U: State>(pos: &Position, movelist: &mut T) {
    let from = pos.us.king;
    let unsafe_squares = pos.unsafe_sq::<U>();

    if (pos.castling_rights & U::ksr_start_sq()).is_not_empty()
        && (U::ksc_mask() & pos.occ).is_empty()
        && (U::ksc_mask() & unsafe_squares).is_empty()
    {
        movelist.add_castle(from, from.east_two(), MoveT::KSCastle);
    }

    if (pos.castling_rights & U::qsr_start_sq()).is_not_empty()
        && (U::qsc_free_mask() & pos.occ).is_empty()
        && (U::qsc_safety_mask() & unsafe_squares).is_empty()
    {
        movelist.add_castle(from, from.west_two(), MoveT::QSCastle);
    }
}

mod pt {
    use super::{types, BitBoard};

    pub trait Piece {
        fn generate_attacks(from: BitBoard, occ: BitBoard) -> BitBoard;
        fn pt() -> types::Piece;
    }

    pub struct Rook;
    pub struct Knight;
    pub struct Bishop;
    pub struct Queen;

    impl Piece for Rook {
        fn generate_attacks(from: BitBoard, occ: BitBoard) -> BitBoard {
            from.rook_magic_lu(occ)
        }
        fn pt() -> types::Piece {
            types::Piece::Rook
        }
    }

    impl Piece for Knight {
        fn generate_attacks(from: BitBoard, _occ: BitBoard) -> BitBoard {
            from.knight_attacks_lu()
        }
        fn pt() -> types::Piece {
            types::Piece::Knight
        }
    }

    impl Piece for Bishop {
        fn generate_attacks(from: BitBoard, occ: BitBoard) -> BitBoard {
            from.bishop_magic_lu(occ)
        }
        fn pt() -> types::Piece {
            types::Piece::Bishop
        }
    }

    impl Piece for Queen {
        fn generate_attacks(from: BitBoard, occ: BitBoard) -> BitBoard {
            from.queen_magic_lu(occ)
        }
        fn pt() -> types::Piece {
            types::Piece::Queen
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    use constants::fen::*;
    use movelist::MoveVec;

    struct Expected {
        count: usize,
        captures: usize,
        pawn: usize,
        knight: usize,
        king: usize,
        castles: usize,
    }

    const START_EXPECTED: Expected = Expected {
        count: 20,
        captures: 0,
        pawn: 16,
        knight: 4,
        king: 0,
        castles: 0,
    };

    const TEST_2_EXPECTED: Expected = Expected {
        count: 48,
        captures: 8,
        pawn: 8,
        knight: 11,
        king: 4,
        castles: 2,
    };

    #[test_case(STARTING_FEN, START_EXPECTED; "start_position")]
    #[test_case(TEST_2, TEST_2_EXPECTED; "position_two")]
    fn test_move_generation(fen: &str, expected: Expected) {
        let pos = Position::from_fen(fen).unwrap();
        let mut movelist = MoveVec::new();
        generate_all(&pos, &mut movelist);
        let count = movelist.len();
        let mut n_captures = 0;
        let mut n_pawn = 0;
        let mut n_knight = 0;
        let mut n_king = 0;
        let mut n_castles = 0;
        for mv in movelist.iter() {
            let moved_pt = pos.us.pt_at(mv.from()).unwrap();
            match moved_pt {
                types::Piece::Pawn => n_pawn += 1,
                types::Piece::Knight => n_knight += 1,
                types::Piece::King => n_king += 1,
                _ => (),
            }
            if matches!(mv.mt(), MoveT::KSCastle | MoveT::QSCastle) {
                n_castles += 1;
            }
            if mv.is_capture() {
                n_captures += 1;
            }
        }
        let mut fails = Vec::new();
        if count != expected.count {
            fails.push(format!(
                "(count: expected {}, found {})",
                expected.count, count
            ))
        }
        if n_captures != expected.captures {
            fails.push(format!(
                "(captures: expected {}, found {})",
                expected.captures, n_captures
            ));
        }
        if n_pawn != expected.pawn {
            fails.push(format!(
                "(pawn: expected {}, found {})",
                expected.pawn, n_pawn
            ));
        }
        if n_knight != expected.knight {
            fails.push(format!(
                "(knight: expected {}, found {})",
                expected.knight, n_knight
            ));
        }
        if n_king != expected.king {
            fails.push(format!(
                "(king: expected {}, found {})",
                expected.king, n_king
            ));
        }
        if n_castles != expected.castles {
            fails.push(format!(
                "(castles: expected {}, found {})",
                expected.castles, n_castles
            ));
        }
        assert!(fails.len() == 0, "{}", fails.join(" "))
    }
}
