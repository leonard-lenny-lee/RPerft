use super::*;
use position::Data;

impl Data {

    pub fn evaluate(&self) -> i32 {
        self.material()
    }

    /// Calculate a game phase value to allow interpolation of middlegame and
    /// endgame phases. Middlegame 24 -> 0 Endgame
    fn _game_phase(&self) -> i32 {
        const KNIGHT: i32 = 1;
        const BISHOP: i32 = 1;
        const ROOK: i32 = 2;
        const QUEEN: i32 = 4;
        const TOTAL: i32 = 24;
        // If phase is > 24, due to promotion, return phase at maximum value of 24
        std::cmp::min(
              KNIGHT * self.knight_sum()
            + BISHOP * self.bishop_sum()
            + ROOK * self.rook_sum()
            + QUEEN * self.queen_sum(),
            TOTAL
        )
    }

    pub fn material(&self) -> i32 {
        const QUEEN: i32 = 1000;
        const ROOK: i32 = 525;
        const BISHOP: i32 = 350;
        const KNIGHT: i32 = 350;
        const PAWN: i32 = 100;
          QUEEN * self.queen_diff()
        + ROOK * self.rook_diff()
        + BISHOP * self.bishop_diff()
        + KNIGHT * self.knight_diff()
        + PAWN * self.pawn_diff()
    }
}

