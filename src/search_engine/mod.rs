use crate::game;
use crate::mechanics::Maps;

pub fn perft(mut pos: &game::Position, depth: i8, maps: &Maps) -> i32 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let moves = pos.generate_moves(maps);
    for mv in moves {
        pos.make_move(&mv);
        nodes += perft(pos, depth-1, maps);
        pos.unmake_move(&mv);
    }
    return nodes;
}