use super::*;

use position::Position;

const MAX_EVALUATION: i32 = i16::MAX as i32;
const MIN_EVALUATION: i32 = i16::MIN as i32;

impl Position {
    pub fn evaluate(&self) -> i16 {
        globals::NN
            .evaluate(
                self.nnue_pos.player,
                self.nnue_pos.pieces(),
                self.nnue_pos.squares(),
            )
            .clamp(MIN_EVALUATION, MAX_EVALUATION) as i16
    }
}
