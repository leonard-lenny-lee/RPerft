use super::*;
use weights::*;
use position::Position;

mod weights;

#[derive(Clone, Copy)]
pub struct Evaluation {
    phase: i32,
    material: i32,
    mg_pst: i32,
    eg_pst: i32,
    who_to_move: i32
}

impl Evaluation {

    fn new() -> Self {
        Evaluation {
            phase: 24,
            material: 0,
            mg_pst: 0,
            eg_pst: 0,
            who_to_move: 1
        }
    }

    /// Create a new evaluation struct based on a position
    pub fn new_from_position(pos: &Position) -> Self {
        let mut eval = Self::new();
        eval.calculate_phase(pos);
        eval.calculate_material_score(pos);
        eval.calculate_pst_scores(pos);
        eval
    }

    /// Calculate a game phase value to allow interpolation of middlegame and
    /// endgame phases. Middlegame 24 -> 0 Endgame
    fn calculate_phase(&mut self, pos: &Position) {
        // If phase is > 24, due to promotion, set phase at maximum value of 24
        self.phase = std::cmp::min(
            phases::KNIGHT * pos.data.knight_sum()
            + phases::BISHOP * pos.data.bishop_sum()
            + phases::ROOK * pos.data.rook_sum()
            + phases::QUEEN * pos.data.queen_sum(),
            phases::TOTAL
        )
    }

    /// Calculate the material balance of the position
    fn calculate_material_score(&mut self, pos: &Position) {
        self.material = material::QUEEN * pos.data.queen_diff()
                        + material::ROOK * pos.data.rook_diff()
                        + material::BISHOP * pos.data.bishop_diff()
                        + material::KNIGHT * pos.data.knight_diff()
                        + material::PAWN * pos.data.pawn_diff()
    }

    /// Evaluate based on the positions of the pieces on the board via
    /// pst weightings
    fn calculate_pst_scores(&mut self, pos: &Position) {
        let w_array = pos.data.w_pieces.as_pst_array();
        let b_array = pos.data.b_pieces.as_pst_array();
        let mut mg_pst = 0;
        let mut eg_pst = 0;
        // Loop through every piece bitboard to evaluate the piece positioning
        for bb_index in 0..6 {
            let mut w_pieces = w_array[bb_index];
            // Pop bits from the bitboards and use index positions to lookup the
            // relevant score from the piece square table
            while w_pieces != common::EMPTY_BB {
                let bit_index = w_pieces.pop_ils1b();
                mg_pst += psts::MG_TABLES[bb_index][bit_index];
                eg_pst += psts::EG_TABLES[bb_index][bit_index];
            }
            // Flip black pieces so their positions align with the map indices
            let mut b_pieces = b_array[bb_index].flip_vertical();
            while b_pieces != common::EMPTY_BB {
                let bit_index = b_pieces.pop_ils1b();
                mg_pst -= psts::MG_TABLES[bb_index][bit_index];
                eg_pst -= psts::EG_TABLES[bb_index][bit_index];
            }
        }
        self.mg_pst = mg_pst;
        self.eg_pst = eg_pst;
    }

    /// Return the overall evaluation of the position by aggegating its
    /// components
    pub fn get_eval(&self) -> i32 {
        (self.material
        + (self.mg_pst * self.phase + self.eg_pst * (phases::TOTAL - self.phase)) / phases::TOTAL)
        * self.who_to_move
    }

    pub fn update_eval(&self, pos: &Position, mv: &search::Move) -> Self {
        Evaluation::new()
    }

}
