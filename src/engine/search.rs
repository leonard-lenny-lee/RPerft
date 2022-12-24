use super::*;
use position::Position;


const NEGATIVE_INFINITY: i32 = -1000000;


/// Search a position for the best evaluation using the exhaustative depth
/// first negamax algorithm. Not to be used in release; use as a testing tool
/// to ensure the same results are reached by alpha beta pruning
/// 
pub fn nega_max(pos: &Position, depth: i8) -> i32 {
    if depth == 0 {
        return pos.data.evaluate()
    }
    let move_list = pos.find_moves();
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    let mut max_evaluation = NEGATIVE_INFINITY;
    for mv in move_list.iter() {
        let new_pos = pos.make_move(mv);
        let evaluation = -nega_max(&new_pos, depth - 1);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
        }
    }
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(
    pos: &Position, depth: i8, mut alpha: i32, beta: i32
) -> i32 {
    if depth == 0 {
        return pos.data.evaluate()
    }
    let move_list = pos.find_moves();
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY // Checkmate
        } else {
            return 0 // Stalemate
        }
    }
    for mv in move_list.iter() {
        let new_pos = pos.make_move(mv);
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

pub mod perft {

    use super::*;
    use transposition::PerftTable;
    use global::Global;

    pub fn perft(pos: &Position, depth: i8, global: &Global) -> (i64, f64, f64) {
        assert!(depth >= 1);
        let mut table = PerftTable::new(global.table_size);
        let start = std::time::Instant::now();
        let nodes = if global.hashing_enabled {
            perft_inner_with_table(pos, depth, &mut table)
        } else {
            perft_inner(pos, depth)
        };
        let duration = start.elapsed().as_secs_f64();
        let nodes_per_second = nodes as f64 / (duration * 1_000_000.0);
        return (nodes, duration, nodes_per_second)
    }

    fn perft_inner(pos: &Position, depth: i8) -> i64 {
        let mut nodes = 0;
        if depth == 1 {
            return pos.find_moves().len() as i64;
        }
        let move_list = pos.find_moves();
        for mv in move_list.iter() {
            let new_pos = pos.make_move(mv);
            nodes += perft_inner(&new_pos, depth-1);
        }
        return nodes
    }

    fn perft_inner_with_table(
        pos: &Position, depth: i8, table: &mut PerftTable
    ) -> i64 {
        let mut nodes = 0;
        if let Some(entry) = table.get(pos.key.0, depth) {
            return entry.count
        };
        if depth == 1 {
            return pos.find_moves().len() as i64;
        }
        let move_list = pos.find_moves();
        for mv in move_list.iter() {
            let new_pos = pos.make_move(mv);
            nodes += perft_inner_with_table(&new_pos, depth-1, table);
        }
        table.set(pos.key.0, nodes, depth);
        return nodes
    }

    /// Provides the number of nodes for down each branch of the first depth layer
    /// search. Useful for perft debugging purposes
    pub fn perft_divided(pos: &Position, depth: i8, global: &Global) -> i64 {
        assert!(depth >= 1);
        let start = std::time::Instant::now();
        let mut nodes = 0;
        let move_list = pos.find_moves();
        let mut table = PerftTable::new(global.table_size);
        for mv in move_list.iter() {
            let new_pos = pos.make_move(mv);
            let branch_nodes;
            if depth == 1 {
                branch_nodes = 1
            } else {
                branch_nodes = if global.hashing_enabled {
                    perft_inner_with_table(&new_pos, depth - 1, &mut table)
                } else {
                    perft_inner(&new_pos, depth - 1)
                }
            }
            // Report branch
            let src = mv.src().to_algebraic();
            let target = mv.target().to_algebraic();
            let mut promotion_piece = "";
            if mv.is_promotion() {
                match mv.promotion_piece() {
                    2 => promotion_piece = "r",
                    3 => promotion_piece = "n",
                    4 => promotion_piece = "b",
                    5 => promotion_piece = "q",
                    _ => ()
                }
            }
            println!("{}{}{}: {}", src, target, promotion_piece, branch_nodes);
            nodes += branch_nodes;
        }
        // Report perft results
        let duration = start.elapsed().as_secs_f64();
        let nodes_per_second = nodes as f64 / (duration * 1_000_000.0);
        println!("{} nodes found in {:.2} seconds ({:.1}M/s)", nodes, duration, nodes_per_second);
        return nodes
    }

    macro_rules! run_suite {
        ($n_tests: ident, $positions: ident, $depths: ident, $global: ident) => {
            $global.report_config();
            for i in 0..$n_tests {
                let pos = Position::from_fen($positions[i].to_string()).unwrap();
                let (nodes, duration, nodes_per_second) = perft(&pos, $depths[i], &$global);
                println!(
                    "Test #{}: {} nodes in {:.2} seconds ({:.2} M/s)",
                    i + 1, nodes, duration, nodes_per_second 
                )
            }
        };
    }

    pub fn run_perft_bench() {
        let mut global = Global::init();

        let positions = [DEFAULT_FEN, POSITION_2, POSITION_3, POSITION_4, POSITION_5, POSITION_6];
        let depths = [6, 5, 7, 5, 5, 5];

        assert_eq!(positions.len(), depths.len());
        let n_tests = positions.len();

        global.hashing_enabled = false;
        run_suite!(n_tests, positions, depths, global);
        global.hashing_enabled = true;
        run_suite!(n_tests, positions, depths, global);

    }

}