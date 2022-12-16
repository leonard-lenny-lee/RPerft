use crate::engine::*;
use common::*;
use bittools as bt;

pub mod depth_search;
pub mod move_generation;
pub mod apply_move;
pub mod move_;

pub use move_::Move;
use evaluation::Evaluation;
use position::Position;

pub struct SearchNode {
    pub pos: Position,
    eval: Evaluation,
    hash: u64,
}

impl SearchNode {
    pub fn new_from_fen(fen: String) -> SearchNode {
        let pos = Position::new_from_fen(fen);
        let eval = Evaluation::new_from_position(&pos);
        let hash = pos.zobrist_hash(&POLYGLOT_RANDOM_ARRAY);
        return SearchNode {pos, eval, hash}
    } 
}