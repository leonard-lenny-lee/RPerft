use crate::engine::*;
use common::*;

pub mod depth_search;
pub mod move_generation;
pub mod apply_move;
pub mod move_;

pub use move_::{Move, MoveList};
use evaluation::Evaluation;
use position::{Position, ZobristKey};

pub struct SearchNode {
    pub pos: Position,
    eval: Evaluation,
    pub key: ZobristKey,
}

impl SearchNode {
    pub fn new_from_fen(fen: String) -> SearchNode {
        let pos = Position::new_from_fen(fen);
        let eval = Evaluation::new_from_position(&pos);
        let key = ZobristKey::new(&pos);
        return SearchNode {pos, eval, key}
    } 
}