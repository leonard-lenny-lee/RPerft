use super::*;

use nnue::NNUE;
use position::Position;

const MAX_EVALUATION: i32 = i16::MAX as i32;
const MIN_EVALUATION: i32 = i16::MIN as i32;

/*
    NNUE Evaluation Procedure
*/
lazy_static! {
    static ref NN: NNUE = NNUE::init(NNUE_PATH);
}

impl Position {
    pub fn evaluate(&self) -> i16 {
        NN.evaluate(
            self.nnue_pos.player,
            self.nnue_pos.pieces(),
            self.nnue_pos.squares(),
        )
        .clamp(MIN_EVALUATION, MAX_EVALUATION) as i16
    }
}
