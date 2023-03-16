use super::*;
use position::Position;

pub fn evaluate(pos: &Position) -> i16 {
    material(pos)
}

/// Calculate a game phase value to allow interpolation of middlegame and
/// endgame phases. Middlegame 24 -> 0 Endgame
fn game_phase(pos: &Position) -> i16 {
    const KNIGHT: i16 = 1;
    const BISHOP: i16 = 1;
    const ROOK: i16 = 2;
    const QUEEN: i16 = 4;
    const TOTAL: i16 = 24;

    let phase = KNIGHT * (pos.white.knight.pop_count() + pos.black.knight.pop_count())
        + BISHOP * (pos.white.bishop.pop_count() + pos.black.bishop.pop_count())
        + ROOK * (pos.white.rook.pop_count() + pos.black.rook.pop_count())
        + QUEEN * (pos.white.queen.pop_count() + pos.black.queen.pop_count());

    // If phase is > 24, due to promotion, return phase at maximum value of 24
    return std::cmp::min(phase, TOTAL);
}

fn material(pos: &Position) -> i16 {
    const QUEEN: i16 = 1000;
    const ROOK: i16 = 525;
    const BISHOP: i16 = 350;
    const KNIGHT: i16 = 350;
    const PAWN: i16 = 100;

    KNIGHT * (pos.white.knight.pop_count() - pos.black.knight.pop_count())
        + BISHOP * (pos.white.bishop.pop_count() - pos.black.bishop.pop_count())
        + ROOK * (pos.white.rook.pop_count() - pos.black.rook.pop_count())
        + QUEEN * (pos.white.queen.pop_count() - pos.black.queen.pop_count())
        + PAWN * (pos.white.pawn.pop_count() - pos.black.pawn.pop_count())
}
