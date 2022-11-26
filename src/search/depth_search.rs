use crate::position::{Position, states::Pos};
use crate::global::maps::Maps;
use super::move_generation::{generate_moves, apply_move};

pub fn perft(pos: &impl Pos, depth: i8, maps: &Maps) -> i32 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let moves = generate_moves(pos, maps);
    for mv in moves {
        let new_pos = apply_move(pos, &mv);
        nodes += perft(new_pos, depth-1, maps);
    }
    return nodes;
}