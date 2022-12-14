use super::*;
use common::bittools as bt;
use weights::*;
use position::Position;

mod weights;

pub fn evaluate(pos: &Position) -> i32 {
    let who_to_move;
    if pos.data.white_to_move {
        who_to_move = 1
    } else {
        who_to_move = -1
    }
    let phase = calculate_game_phase(pos);
    return (
        material_score(pos) 
        + piece_square_table_score(pos, phase)
    ) * who_to_move
}

/// Evaluate based on the material on the board
fn material_score(pos: &Position) -> i32 {
    material::QUEEN * pos.data.queen_diff()
    + material::ROOK * pos.data.rook_diff()
    + material::BISHOP * pos.data.bishop_diff()
    + material::KNIGHT * pos.data.knight_diff()
    + material::PAWN * pos.data.pawn_diff()
}

/// Evaluate based on the positions of the pieces on the board via pst weightings
fn piece_square_table_score(pos: &Position, phase: i32) -> i32 {
    let w_array = pos.data.w_pieces.as_pst_array();
    let b_array = pos.data.b_pieces.as_pst_array();
    let mut mg_score = 0;
    let mut eg_score = 0;
    // Loop through every piece bitboard to evaluate the piece positioning
    for bb_index in 0..6 {
        let mut w_pieces = w_array[bb_index];
        // Pop bits from the bitboards and use index positions to lookup the
        // relevant score from the piece square table
        while w_pieces != common::EMPTY_BB {
            let bit_index = bt::pop_ilsb(&mut w_pieces);
            mg_score += psts::MG_TABLES[bb_index][bit_index];
            eg_score += psts::EG_TABLES[bb_index][bit_index];
        }
        // Flip black pieces so their positions align with the map indices
        let mut b_pieces = bt::flip_vertical(b_array[bb_index]);
        while b_pieces != common::EMPTY_BB {
            let bit_index = bt::pop_ilsb(&mut b_pieces);
            mg_score -= psts::MG_TABLES[bb_index][bit_index];
            eg_score -= psts::EG_TABLES[bb_index][bit_index];
        }
    }
    (mg_score * phase + eg_score * (phases::TOTAL - phase)) / phases::TOTAL
}

/// Calculate a game phase value to allow interpolation of middlegame and
/// endgame phases; a value of 24 indicates the pure middlegame and a value 
/// of 0 indicated the pure endgame
fn calculate_game_phase(pos: &Position) -> i32 {
    // If game phase is > 24, for example if there is an early promotion, use
    // the maximum phase value of 24 instead
    std::cmp::min(
        phases::KNIGHT * pos.data.knight_sum()
        + phases::BISHOP * pos.data.bishop_sum()
        + phases::ROOK * pos.data.rook_sum()
        + phases::QUEEN * pos.data.queen_sum(),
        phases::TOTAL
    )
}