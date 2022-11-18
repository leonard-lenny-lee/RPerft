use crate::position::Position;
use crate::global::maps::Maps;
use super::move_generation::generate_moves;

pub fn perft(pos: &mut Position, depth: i8, maps: &Maps) -> i32 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let moves = generate_moves(pos, maps);
    for mv in moves {
        pos.make_move(&mv);
        nodes += perft(pos, depth-1, maps);
        pos.unmake_move(&mv);
    }
    return nodes;
}