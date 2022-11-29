use super::*;
use position::{Position, analysis_tools::find_checkers};
use global::maps::Maps;
use move_generation::{find_moves, apply_move::apply_move};
use evaluation::evaluate;

const NEGATIVE_INFINITY: i32 = -1000000;

/// Search a position for the best evaluation using the exhaustative depth
/// first negamax algorithm. Not to be used in release; use as a testing tool
/// to ensure the same results are reached by alpha beta pruning
pub fn nega_max(pos: &Position, depth: i8, maps: &Maps) -> i32 {
    if depth == 0 {
        return evaluate(pos)
    }
    let moves = find_moves(pos, maps);
    if moves.len() == 0 {
        let n_checkers = find_checkers(pos, maps).count_ones();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    let mut max_evaluation = NEGATIVE_INFINITY;
    for mv in moves {
        let new_pos = apply_move(pos, &mv);
        let evaluation = -nega_max(&new_pos, depth - 1, maps);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
        }
    }
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(pos: &Position, depth: i8, maps: &Maps,
              mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(pos)
    }
    let moves = find_moves(pos, maps);
    if moves.len() == 0 {
        let n_checkers = find_checkers(pos, maps).count_ones();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    for mv in find_moves(pos, maps) {
        let new_pos = apply_move(pos, &mv);
        let evaluation = -alpha_beta(
            &new_pos, depth - 1, maps, -alpha, -beta
        );
        if evaluation >= beta {
            return beta // Pruning condition
        }
        if evaluation > alpha {
            alpha = evaluation
        }
    }
    return alpha
}
