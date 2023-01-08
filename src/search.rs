use super::*;
use config::{Config, SearchMethod};
use evaluate::evaluate;
use makemove::make_move;
use movegen::{find_captures, find_check_evasions, find_moves};
use movelist::Move;
use position::Position;
use transposition::{HashTable, SearchEntry, SharedHashTable};

const NEGATIVE_INFINITY: i32 = -1000000;
const POSITIVE_INFINITY: i32 = 1000000;

#[derive(Clone, Copy)]
pub enum NodeType {
    PV,
    Cut,
    All,
}

pub fn do_search(
    config: &mut Config,
    pos: &Position,
    depth: u8,
    table: &mut HashTable<SearchEntry>,
) {
    // Execute search
    match config.search_method {
        SearchMethod::Negamax => nega_max(pos, depth, table),
        SearchMethod::AlphaBeta => {
            alpha_beta(pos, depth, NEGATIVE_INFINITY, POSITIVE_INFINITY, table)
        }
    };

    // Probe table for the results of the search
    if let Some(entry) = table.get(pos.key.0, depth) {
        let pv = probe_pv(pos, depth, table);
        let pv_algebraic = pv
            .into_iter()
            .map(|m| m.to_algebraic())
            .collect::<Vec<String>>()
            .join(" ");
        println!(
            "bestmove {} {} pv {}",
            entry.best_move.to_algebraic(),
            entry.evaluation,
            pv_algebraic,
        );
    };
}

fn probe_pv(pos: &Position, depth: u8, table: &mut HashTable<SearchEntry>) -> Vec<Move> {
    let mut pos = pos.clone();
    let mut depth = depth;
    let mut pv = Vec::new();
    while depth > 0 {
        if let Some(entry) = table.get(pos.key.0, depth) {
            if !entry.best_move.is_null() {
                pos = make_move(&pos, &entry.best_move);
                pv.push(entry.best_move);
            }
        } else {
            break;
        }
        depth -= 1;
    }
    return pv;
}

/// Search a position for the best evaluation using the exhaustative depth
/// first negamax algorithm. Not to be used in release; use as a testing tool
/// to ensure the same results are reached by alpha beta pruning
pub fn nega_max(pos: &Position, depth: u8, table: &mut HashTable<SearchEntry>) -> i32 {
    if let Some(entry) = table.get(pos.key.0, depth) {
        return entry.evaluation;
    }
    if depth == 0 {
        return evaluate(pos);
    }
    let move_list = find_moves(pos);
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    let mut best_move = movelist::Move::new_null();
    let mut max_evaluation = NEGATIVE_INFINITY;
    for mv in move_list.iter() {
        let new_pos = make_move(pos, mv);
        let evaluation = -nega_max(&new_pos, depth - 1, table);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
            best_move = *mv;
        }
    }
    table.set(SearchEntry {
        key: pos.key.0,
        depth,
        best_move,
        evaluation: max_evaluation,
        node_type: NodeType::PV,
    });
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(
    pos: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    table: &mut HashTable<SearchEntry>,
) -> i32 {
    if let Some(entry) = table.get(pos.key.0, depth) {
        return entry.evaluation;
    }
    if depth == 0 {
        return quiesce(pos, alpha, beta, 0);
    }
    let move_list = find_moves(pos);
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return NEGATIVE_INFINITY; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    let mut best_move = movelist::Move::new_null();
    let mut is_pv = false;
    for mv in move_list.iter() {
        let new_pos = make_move(pos, mv);
        let evaluation = -alpha_beta(&new_pos, depth - 1, -beta, -alpha, table);
        if evaluation >= beta {
            table.set(SearchEntry {
                key: pos.key.0,
                depth,
                best_move,
                evaluation: beta,
                node_type: NodeType::Cut,
            });
            return beta; // Pruning condition
        }
        if evaluation > alpha {
            alpha = evaluation;
            is_pv = true;
            best_move = *mv;
        }
    }
    table.set(SearchEntry {
        key: pos.key.0,
        depth,
        best_move,
        evaluation: alpha,
        node_type: if is_pv { NodeType::PV } else { NodeType::All },
    });
    return alpha;
}

fn quiesce(pos: &Position, mut alpha: i32, beta: i32, ply: i8) -> i32 {
    let stand_pat = evaluate(pos);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    let checkers = pos.find_checkers();
    let target_squares = pos.target_squares(); // All squares our pieces are attacking
    let possible_captures = target_squares & pos.their_pieces().any;
    let move_list = if checkers != EMPTY_BB {
        // If in check, the priority is to resolve the check
        let move_list = find_check_evasions(pos, checkers);
        if move_list.len() == 0 {
            // Checkmate
            return NEGATIVE_INFINITY;
        }
        move_list
    } else if possible_captures != EMPTY_BB {
        // Enumerate the through the captures only
        find_captures(pos)
    } else {
        // No captures and not in check
        return alpha;
    };

    for mv in move_list.iter() {
        let new_pos = make_move(pos, mv);
        let score = -quiesce(&new_pos, -beta, -alpha, ply + 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score
        }
    }

    return alpha;
}

pub mod perft {

    use super::*;
    use config::PerftConfig;
    use std::sync::{mpsc::channel, Arc};
    use threadpool::ThreadPool;
    use transposition::{PerftEntry, SharedPerftEntry};

    pub fn perft(pos: &Position, depth: u8, config: &PerftConfig) -> (u64, f64, f64) {
        assert!(depth >= 1);
        let start = std::time::Instant::now();
        let nodes = if config.multithreading {
            perft_multithreaded(pos, depth, config.hashing, config, false)
        } else {
            let mut table = if config.hashing {
                Some(HashTable::new(config.table_size))
            } else {
                None
            };
            perft_inner(pos, depth, &mut table, config)
        };
        let duration = start.elapsed().as_secs_f64();
        let nodes_per_second = nodes as f64 / (duration * 1_000_000.0);
        return (nodes, duration, nodes_per_second);
    }

    fn perft_multithreaded(
        pos: &Position,
        depth: u8,
        hashing: bool,
        config: &PerftConfig,
        verbose: bool,
    ) -> u64 {
        let moves = find_moves(pos);
        let config = config.clone();
        let n_jobs = moves.len();
        let pool = ThreadPool::new(config.num_threads);
        let (tx, rx) = channel();
        let table = Arc::new(SharedHashTable::new(config.table_size));
        for i in 0..n_jobs {
            let tx = tx.clone();
            let mv = moves[i];
            let new_pos = make_move(pos, &mv);
            let table = if hashing { Some(table.clone()) } else { None };
            pool.execute(move || {
                let count = perft_inner_shared_hash_table(&new_pos, depth - 1, &table, &config);
                tx.send(count).unwrap();
                if verbose {
                    println!("{}: {}", mv.to_algebraic(), count);
                }
            });
        }
        return rx.iter().take(n_jobs).fold(0, |a, b| a + b);
    }

    fn perft_inner(
        pos: &Position,
        depth: u8,
        table: &mut Option<HashTable<PerftEntry>>,
        config: &PerftConfig,
    ) -> u64 {
        let mut nodes = 0;
        if let Some(table) = table {
            if let Some(entry) = table.get(pos.key.0, depth) {
                return entry.count;
            };
        }
        if depth == 1 && config.bulk_counting {
            return find_moves(pos).len() as u64;
        }
        if depth == 0 {
            return 1;
        }
        let move_list = find_moves(pos);
        for mv in move_list.iter() {
            let new_pos = make_move(pos, mv);
            nodes += perft_inner(&new_pos, depth - 1, table, config);
        }
        if let Some(table) = table {
            table.set(PerftEntry {
                key: pos.key.0,
                count: nodes,
                depth,
            });
        }
        return nodes;
    }

    fn perft_inner_shared_hash_table(
        pos: &Position,
        depth: u8,
        table: &Option<Arc<SharedHashTable<SharedPerftEntry>>>,
        config: &PerftConfig,
    ) -> u64 {
        let mut nodes = 0;
        if let Some(table) = table {
            if let Some(entry) = table.get(pos.key.0, depth) {
                return entry.count().into();
            };
        }
        if depth == 1 && config.bulk_counting {
            return find_moves(pos).len() as u64;
        }
        if depth == 0 {
            return 1;
        }
        let move_list = find_moves(pos);
        for mv in move_list.iter() {
            let new_pos = make_move(pos, mv);
            nodes += perft_inner_shared_hash_table(&new_pos, depth - 1, table, config);
        }
        if let Some(table) = table {
            table.set(SharedPerftEntry::new(pos.key.0, depth, nodes), pos.key.0);
        }
        return nodes;
    }

    /// Provides the number of nodes for down each branch of the first depth
    /// search. Useful for perft debugging purposes
    pub fn perft_divided(pos: &Position, depth: u8, config: &PerftConfig) -> u64 {
        assert!(depth >= 1);
        let start = std::time::Instant::now();
        // Perft generally runs faster with hashing at higher depths
        let nodes = perft_multithreaded(pos, depth, config.hashing, config, true);
        // Report perft results
        let duration = start.elapsed().as_secs_f64();
        let nodes_per_second = nodes as f64 / (duration * 1_000_000.0);
        println!(
            "Nodes searched: {}\nTime elapsed: {:.2} s ({:.1} M/s)",
            nodes, duration, nodes_per_second
        );
        return nodes;
    }

    macro_rules! run_suite {
        ($n_tests: ident, $positions: ident, $depths: ident, $config: ident) => {
            $config.report_config();
            let mut results = Vec::new();
            for i in 0..$n_tests {
                let pos = Position::from_fen($positions[i].to_string()).unwrap();
                let (nodes, duration, nodes_per_second) = perft(&pos, $depths[i], &$config);
                results.push((i + 1, nodes, duration, nodes_per_second));
            }
            println!("+{}+", "-".repeat(34));
            println!(
                "|{:>3} |{:>11} |{:>6} |{:>7} |",
                "#", "Nodes", "sec", "MN/s"
            );
            println!("+{}+", "-".repeat(34));
            for (n, nodes, duration, nodes_per_second) in results {
                println!(
                    "|{:>3} |{:>11} |{:>6} |{:>7} |",
                    n,
                    nodes,
                    format!("{:.2}", duration),
                    format!("{:.2}", nodes_per_second)
                )
            }
            println!("+{}+", "-".repeat(34))
        };
    }

    pub fn run_perft_bench() {
        let mut config = PerftConfig::initialize();

        let positions = [
            DEFAULT_FEN,
            POSITION_2,
            POSITION_3,
            POSITION_4,
            POSITION_5,
            POSITION_6,
        ];
        let depths = [6, 5, 7, 5, 5, 5];

        assert_eq!(positions.len(), depths.len());
        let n_tests = positions.len();
        println!("Running Perft Suite...");
        (config.multithreading, config.hashing) = (true, true);
        run_suite!(n_tests, positions, depths, config);
        (config.multithreading, config.hashing) = (true, false);
        run_suite!(n_tests, positions, depths, config);
        config.bulk_counting = false;
        run_suite!(n_tests, positions, depths, config);
    }
}
