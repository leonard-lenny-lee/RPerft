use super::*;
use weights::*;
use position::Position;

mod weights;

pub fn evaluate(pos: &Position) -> i32 {
    return material_score(pos)
}

/// Evaluate based on the material on the board
fn material_score(pos: &Position) -> i32 {
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    material::KING * (our_pieces.n_kings() - their_pieces.n_kings())
    + material::QUEEN * (our_pieces.n_queens() - their_pieces.n_queens())
    + material::ROOK * (our_pieces.n_rooks() - their_pieces.n_rooks())
    + material::BISHOP * (our_pieces.n_bishops() - their_pieces.n_bishops())
    + material::KNIGHT * (our_pieces.n_knights() - their_pieces. n_knights())
    + material::PAWN * (our_pieces.n_pawns() - their_pieces.n_pawns())
}