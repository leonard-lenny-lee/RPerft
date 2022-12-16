use super::*;
use position::{Position, analysis_tools::find_checkers};
use move_generation::find_moves;
use apply_move::apply_move;

const NEGATIVE_INFINITY: i32 = -1000000;

/// Search a position for the best evaluation using the exhaustative depth
/// first negamax algorithm. Not to be used in release; use as a testing tool
/// to ensure the same results are reached by alpha beta pruning
pub fn nega_max(node: &SearchNode, depth: i8) -> i32 {
    if depth == 0 {
        return node.eval.get_eval()
    }
    let moves = find_moves(&node.pos);
    if moves.len() == 0 {
        let n_checkers = find_checkers(&node.pos).count_ones();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    let mut max_evaluation = NEGATIVE_INFINITY;
    for mv in moves {
        let new_node = apply_move(&node, &mv);
        let evaluation = -nega_max(&new_node, depth - 1);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
        }
    }
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(
    node: &SearchNode, depth: i8, mut alpha: i32, beta: i32
) -> i32 {
    if depth == 0 {
        return node.eval.get_eval()
    }
    let moves = find_moves(&node.pos);
    if moves.len() == 0 {
        let n_checkers = find_checkers(&node.pos).count_ones();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    for mv in moves {
        let new_pos = apply_move(node, &mv);
        let evaluation = -alpha_beta(
            &new_pos, depth - 1, -alpha, -beta
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

// #[cfg(test)]
// mod search_tests {
//     use super::*;
//     #[test]
//     fn test_negamax() {
//         let pos = &Position::new_from_fen(POSITION_2.to_string());
//         let eval = nega_max(pos, 4);
//         println!("Eval {}", eval);
//     }
// }