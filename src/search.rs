use super::*;
use evaluate::evaluate;
use makemove::make_move;
use movegen::find_moves;
use movelist::Move;
use position::Position;
use transposition::{SearchEntry, HashTable, SharedHashTable, SharedPerftEntry};

const NEGATIVE_INFINITY: i32 = -1000000;

pub fn nega_max_search(pos: &Position, depth: u8, table: &mut HashTable<SearchEntry>) {
    // Execute search
    nega_max(pos, depth, table);
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
    });
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(pos: &Position, depth: i8, mut alpha: i32, beta: i32) -> i32 {
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
    for mv in move_list.iter() {
        let new_pos = make_move(pos, mv);
        let evaluation = -alpha_beta(&new_pos, depth - 1, -alpha, -beta);
        if evaluation >= beta {
            return beta; // Pruning condition
        }
        if evaluation > alpha {
            alpha = evaluation
        }
    }
    return alpha;
}

pub mod perft {

    use super::*;
    use config::PerftConfig;
    use std::sync::{Arc, mpsc::channel};
    use threadpool::ThreadPool;
    use transposition::{PerftEntry, HashTable};

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
            table.set(
                SharedPerftEntry::new(pos.key.0, depth, nodes),
                pos.key.0
            );
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
            println!(" {}", "-".repeat(34));
            println!(
                "|{:>3} |{:>11} |{:>6} |{:>7} |",
                "#", "Nodes", "sec", "MN/s"
            );
            println!(" {}", "-".repeat(34));
            for (n, nodes, duration, nodes_per_second) in results {
                println!(
                    "|{:>3} |{:>11} |{:>6} |{:>7} |",
                    n,
                    nodes,
                    format!("{:.2}", duration),
                    format!("{:.2}", nodes_per_second)
                )
            }
            println!(" {}", "-".repeat(34))
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
