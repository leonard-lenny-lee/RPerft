use super::*;
use position::Position;

pub fn evaluate(pos: &Position) -> i16 {
    material(pos)
}

/// Calculate a game phase value to allow interpolation of middlegame and
/// endgame phases. Middlegame 24 -> 0 Endgame
fn _game_phase(pos: &Position) -> i16 {
    const KNIGHT: i16 = 1;
    const BISHOP: i16 = 1;
    const ROOK: i16 = 2;
    const QUEEN: i16 = 4;
    const TOTAL: i16 = 24;
    // If phase is > 24, due to promotion, return phase at maximum value of 24
    std::cmp::min(
        KNIGHT * pos.data.knight_sum()
            + BISHOP * pos.data.bishop_sum()
            + ROOK * pos.data.rook_sum()
            + QUEEN * pos.data.queen_sum(),
        TOTAL,
    )
}

fn material(pos: &Position) -> i16 {
    const QUEEN: i16 = 1000;
    const ROOK: i16 = 525;
    const BISHOP: i16 = 350;
    const KNIGHT: i16 = 350;
    const PAWN: i16 = 100;
    QUEEN * pos.data.queen_diff()
        + ROOK * pos.data.rook_diff()
        + BISHOP * pos.data.bishop_diff()
        + KNIGHT * pos.data.knight_diff()
        + PAWN * pos.data.pawn_diff()
}
