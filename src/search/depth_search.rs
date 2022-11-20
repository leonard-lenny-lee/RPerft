use crate::position::Position;
use crate::global::maps::Maps;
use super::move_generation::{generate_moves, make_move};

pub fn perft(pos: Position, depth: i8, maps: &Maps) -> i32 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let moves = generate_moves(&pos, maps);
    for mv in moves {
        let new_pos = make_move(pos, &mv);
        nodes += perft(new_pos, depth-1, maps);
    }
    return nodes;
}