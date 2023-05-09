use super::*;

use nnue::NNUE;
use position::Position;

/*
    NNUE Evaluation Procedure
*/
lazy_static! {
    static ref NN: NNUE = NNUE::init(NNUE_PATH);
}

impl Position {
    pub fn evaluate(&self) -> i16 {
        const MAX: i32 = i16::MAX as i32;
        const MIN: i32 = i16::MIN as i32;
        return NN
            .evaluate(
                self.nnue_pos.player,
                self.nnue_pos.pieces(),
                self.nnue_pos.squares(),
            )
            .clamp(MIN, MAX) as i16;
    }
}
