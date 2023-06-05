use super::*;

lazy_static! {
    // Neural network
    pub static ref NN: Box<nnue::NNUE> = nnue::NNUE::init(NNUE_FILE);
}
