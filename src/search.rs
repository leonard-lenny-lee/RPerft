use super::*;
use config::{Config, SearchMethod};
use evaluate::evaluate;
use makemove::make_move;
use movegen::{find_captures, find_check_evasions, find_moves};
use movelist::Move;
use position::Position;
use transposition::{HashTable, Probe};

const NEGATIVE_INFINITY: i16 = -30001;
const CHECKMATE: i16 = -30000;
const POSITIVE_INFINITY: i16 = 30000;

#[derive(Clone, Copy)]
pub enum NodeType {
    PV,
    Cut,
    All,
}

pub fn do_search(config: &mut Config, pos: &Position, depth: u8, table: &mut HashTable) {
    table.age += 1;
    // Execute search
    match config.search_method {
        SearchMethod::Negamax => {
            log::info!("Executing NegaMax search...");
            nega_max(pos, depth, table)
        }
        SearchMethod::AlphaBeta => {
            log::info!("Executing AlphaBeta search...");
            alpha_beta(pos, depth, NEGATIVE_INFINITY, POSITIVE_INFINITY, table)
        }
    };

    // Probe table for the results of the search
    if let Probe::Read(entry) = table.probe_search(pos.key.0, depth) {
        let pv = probe_pv(pos, depth, table);
        let pv_algebraic = pv
            .into_iter()
            .map(|m| m.to_algebraic())
            .collect::<Vec<String>>()
            .join(" ");
        println!(
            "bestmove {} {} pv {}",
            entry.best_move.to_algebraic(),
            entry.score,
            pv_algebraic,
        );
    };
}

fn probe_pv(pos: &Position, depth: u8, table: &HashTable) -> Vec<Move> {
    let mut pos = pos.clone();
    let mut depth = depth;
    let mut pv = Vec::new();
    while depth > 0 {
        if let Probe::Read(entry) = table.probe_search(pos.key.0, depth) {
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
pub fn nega_max(pos: &Position, depth: u8, table: &HashTable) -> i16 {
    let probe_result = table.probe_search(pos.key.0, depth);
    if let Probe::Read(entry) = probe_result {
        return entry.score;
    }
    if depth == 0 {
        return evaluate(pos);
    }
    let move_list = find_moves(pos);
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return CHECKMATE; // Checkmate
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
    if let Probe::Write = probe_result {
        table.write_search(pos.key.0, depth, best_move, max_evaluation, NodeType::PV);
    }
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(pos: &Position, depth: u8, mut alpha: i16, beta: i16, table: &HashTable) -> i16 {
    let probe_result = table.probe_search(pos.key.0, depth);
    if let Probe::Read(entry) = probe_result {
        return entry.score;
    }
    if depth == 0 {
        return quiesce(pos, alpha, beta, 0);
    }
    let move_list = find_moves(pos);
    if move_list.len() == 0 {
        let n_checkers = pos.find_checkers().pop_count();
        if n_checkers > 0 {
            return CHECKMATE; // Checkmate
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
            if let Probe::Write = probe_result {
                table.write_search(pos.key.0, depth, best_move, beta, NodeType::Cut);
            }
            return beta; // Pruning condition
        }
        if evaluation > alpha {
            alpha = evaluation;
            is_pv = true;
            best_move = *mv;
        }
    }
    if let Probe::Write = probe_result {
        table.write_search(
            pos.key.0,
            depth,
            best_move,
            alpha,
            if is_pv { NodeType::PV } else { NodeType::All },
        );
    }
    return alpha;
}

fn quiesce(pos: &Position, mut alpha: i16, beta: i16, ply: i8) -> i16 {
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
            return CHECKMATE;
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
    use std::sync::{mpsc::channel, Arc};
    use threadpool::ThreadPool;
    use transposition::HashTable;

    pub fn perft(
        pos: &Position,
        depth: u8,
        num_threads: usize,
        table_size: usize,
        verbose: bool,
    ) -> (u64, f64, f64) {
        let nodes;
        let start = std::time::Instant::now();
        let moves = find_moves(pos);
        if depth == 0 {
            nodes = 1
        } else if depth == 1 {
            nodes = moves.len() as u64
        } else {
            let n_jobs = moves.len();
            let pool = ThreadPool::new(num_threads);
            let (tx, rx) = channel();
            let table = Arc::new(HashTable::new(table_size));
            for i in 0..n_jobs {
                let tx = tx.clone();
                let mv = moves[i];
                let new_pos = make_move(pos, &mv);
                let table = table.clone();
                pool.execute(move || {
                    let node_count = perft_inner(&new_pos, depth - 1, &table);
                    tx.send(node_count).unwrap();
                    if verbose {
                        println!("{}: {}", mv.to_algebraic(), node_count);
                    }
                })
            }
            nodes = rx.iter().take(n_jobs).fold(0, |a, b| a + b);
        }
        let duration = start.elapsed().as_secs_f64();
        let nodes_per_second = nodes as f64 / (duration * 1_000_000.0);
        if verbose {
            println!(
                "Nodes searched: {}\nTime elapsed: {:.2} s ({:.1} M/s)",
                nodes, duration, nodes_per_second
            );
        }
        return (nodes, duration, nodes_per_second);
    }

    fn perft_inner(pos: &Position, depth: u8, table: &Arc<HashTable>) -> u64 {
        let mut nodes = 0;
        if let Some(nodes) = table.probe_perft(pos.key.0, depth) {
            return nodes;
        }
        let move_list = find_moves(pos);
        if depth == 1 {
            return move_list.len() as u64;
        }
        for mv in move_list.iter() {
            let new_pos = make_move(pos, mv);
            nodes += perft_inner(&new_pos, depth - 1, table);
        }
        table.write_perft(pos.key.0, depth, nodes);
        return nodes;
    }

    pub fn run_perft_suite(config: &Config) {
        let positions = [
            DEFAULT_FEN,
            POSITION_2,
            POSITION_3,
            POSITION_4,
            POSITION_5,
            POSITION_6,
        ];
        let depths = [6, 5, 7, 5, 5, 5];
        let mut results = Vec::new();
        for (i, (pos_fen, depth)) in std::iter::zip(positions, depths).enumerate() {
            let pos = Position::from_fen(pos_fen.to_string()).unwrap();
            let (nodes, duration, nodes_per_second) =
                perft(&pos, depth, config.num_threads, config.table_size, false);
            results.push((i + 1, nodes, duration, nodes_per_second));
        }
        // Report results
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
    }
}
