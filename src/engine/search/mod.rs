use crate::engine::*;
use common::*;
use bittools as bt;

pub mod depth_search;
pub mod move_generation;
pub mod apply_move;
pub mod move_;

pub use move_::Move;
use evaluation::Evaluation;
use position::{Position, ZobristHash};

pub struct SearchNode {
    pub pos: Position,
    eval: Evaluation,
    hash: ZobristHash,
}

impl SearchNode {
    pub fn new_from_fen(fen: String) -> SearchNode {
        let pos = Position::new_from_fen(fen);
        let eval = Evaluation::new_from_position(&pos);
        let hash = ZobristHash::hash(&pos);
        return SearchNode {pos, eval, hash}
    } 
}