use super::*;

lazy_static! {
    // Evaluation Neural network
    pub static ref NN: Box<nnue::NNUE> = nnue::NNUE::init(constants::NNUE_FILE);
}
