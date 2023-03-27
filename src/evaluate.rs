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

    let phase = KNIGHT * (pos.us.knight.pop_count() + pos.them.knight.pop_count())
        + BISHOP * (pos.us.bishop.pop_count() + pos.them.bishop.pop_count())
        + ROOK * (pos.us.rook.pop_count() + pos.them.rook.pop_count())
        + QUEEN * (pos.us.queen.pop_count() + pos.them.queen.pop_count());

    // If phase is > 24, due to promotion, return phase at maximum value of 24
    return std::cmp::min(phase, TOTAL);
}

fn material(pos: &Position) -> i16 {
    const QUEEN: i16 = 1000;
    const ROOK: i16 = 525;
    const BISHOP: i16 = 350;
    const KNIGHT: i16 = 350;
    const PAWN: i16 = 100;

    let (w, b) = pos.white_black();

    KNIGHT * (w.knight.pop_count() - b.knight.pop_count())
        + BISHOP * (w.bishop.pop_count() - b.bishop.pop_count())
        + ROOK * (w.rook.pop_count() - b.rook.pop_count())
        + QUEEN * (w.queen.pop_count() - b.queen.pop_count())
        + PAWN * (w.pawn.pop_count() - b.pawn.pop_count())
}
