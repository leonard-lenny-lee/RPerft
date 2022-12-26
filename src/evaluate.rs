use super::*;
use position::Position;

pub fn evaluate(pos: &Position) -> i32 {
    material(pos)
}

/// Calculate a game phase value to allow interpolation of middlegame and
/// endgame phases. Middlegame 24 -> 0 Endgame
fn _game_phase(pos: &Position) -> i32 {
    const KNIGHT: i32 = 1;
    const BISHOP: i32 = 1;
    const ROOK: i32 = 2;
    const QUEEN: i32 = 4;
    const TOTAL: i32 = 24;
    // If phase is > 24, due to promotion, return phase at maximum value of 24
    std::cmp::min(
            KNIGHT * pos.data.knight_sum()
        + BISHOP * pos.data.bishop_sum()
        + ROOK * pos.data.rook_sum()
        + QUEEN * pos.data.queen_sum(),
        TOTAL
    )
}

pub fn material(pos: &Position) -> i32 {
    const QUEEN: i32 = 1000;
    const ROOK: i32 = 525;
    const BISHOP: i32 = 350;
    const KNIGHT: i32 = 350;
    const PAWN: i32 = 100;
    QUEEN * pos.data.queen_diff()
    + ROOK * pos.data.rook_diff()
    + BISHOP * pos.data.bishop_diff()
    + KNIGHT * pos.data.knight_diff()
    + PAWN * pos.data.pawn_diff()
}

