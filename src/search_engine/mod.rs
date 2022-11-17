use crate::game;
use crate::mechanics::Maps;

fn hyp_quint(o: u64, s: u64, m: &[u64; 64]) -> u64 {
    let m = m[s.trailing_zeros() as usize];
    let mut forward: u64 = o & m;
    let mut reverse: u64 = forward.reverse_bits();
    forward = forward.wrapping_sub(2 * s);
    reverse = reverse.wrapping_sub(2 * s.reverse_bits());
    forward ^= reverse.reverse_bits();
    forward &= m;
    return forward;
}

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