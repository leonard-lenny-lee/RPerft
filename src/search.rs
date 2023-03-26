use super::*;
use evaluate::evaluate;
use hash::{HashTable, Probe};
use movegen::{generate, generate_all};
use movelist::{Move, MoveList, OrderedList, UnorderedList};
use position::Position;

const NEGATIVE_INFINITY: i16 = -30001;
const CHECKMATE: i16 = -30000;
const POSITIVE_INFINITY: i16 = 30000;

#[derive(Clone, Copy)]
pub enum NodeType {
    PV,
    Cut,
    All,
}

pub fn do_search(pos: &Position, depth: u8, table: &mut HashTable) {
    table.age += 1;
    // Execute search
    alpha_beta(pos, depth, NEGATIVE_INFINITY, POSITIVE_INFINITY, table);

    // Probe table for the results of the search
    if let Probe::Read(entry) = table.probe_search(pos.key, depth) {
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
    let mut pos = *pos;
    let mut depth = depth;
    let mut pv = Vec::new();
    while depth > 0 {
        if let Probe::Read(entry) = table.probe_search(pos.key, depth) {
            if !entry.best_move.is_null() {
                pos = pos.make_move(&entry.best_move);
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
    let probe_result = table.probe_search(pos.key, depth);

    if let Probe::Read(entry) = probe_result {
        return entry.score;
    }

    if depth == 0 {
        return evaluate(pos);
    }

    let mut movelist = UnorderedList::new();
    generate_all(pos, &mut movelist);

    if movelist.len() == 0 {
        let n_checkers = pos.checkers().pop_count();
        if n_checkers > 0 {
            return CHECKMATE; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    let mut best_move = movelist::Move::new_null();
    let mut max_evaluation = NEGATIVE_INFINITY;
    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
        let evaluation = -nega_max(&new_pos, depth - 1, table);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
            best_move = *mv;
        }
    }
    if let Probe::Write = probe_result {
        table.write_search(pos.key, depth, best_move, max_evaluation, NodeType::PV);
    }
    return max_evaluation;
}

/// Implementation of alpha-beta pruning to search for the best evaluation
pub fn alpha_beta(pos: &Position, depth: u8, mut alpha: i16, beta: i16, table: &HashTable) -> i16 {
    let probe_result = table.probe_search(pos.key, depth);

    if let Probe::Read(entry) = probe_result {
        return entry.score;
    }

    if depth == 0 {
        return quiesce(pos, alpha, beta, 0);
    }

    let mut movelist = OrderedList::new();
    generate_all(pos, &mut movelist);

    if movelist.len() == 0 {
        let n_checkers = pos.checkers().pop_count();
        if n_checkers > 0 {
            return CHECKMATE; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }

    let mut best_move = movelist::Move::new_null();
    let mut is_pv = false;

    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
        let evaluation = -alpha_beta(&new_pos, depth - 1, -beta, -alpha, table);

        if evaluation >= beta {
            if let Probe::Write = probe_result {
                table.write_search(pos.key, depth, best_move, beta, NodeType::Cut);
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
            pos.key,
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

    let mut movelist = OrderedList::new();
    let checkers = pos.checkers();
    let our_attacks = pos.target_squares(); // All squares our pieces are attacking
    let captures = our_attacks & pos.them().all;

    // If in check, the priority is to resolve the check
    if checkers.is_not_empty() {
        generate(types::GenType::Evasions(checkers), pos, &mut movelist);
        if movelist.len() == 0 {
            return CHECKMATE;
        }
    }
    // Enumerate the through the captures only
    else if captures != EMPTY_BB {
        generate(types::GenType::Captures, pos, &mut movelist);
    }
    // No captures and not in check so stop quiescing
    else {
        return alpha;
    };

    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
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

    pub fn perft(
        pos: &Position,
        depth: u8,
        num_threads: usize,
        table_size: usize,
        verbose: bool,
    ) -> (u64, f64, f64) {
        let start = std::time::Instant::now();

        let mut movelist = UnorderedList::new();
        generate_all(&pos, &mut movelist);

        let nodes = if depth == 0 {
            1
        } else if depth == 1 {
            movelist.len() as u64
        } else {
            let n_jobs = movelist.len();
            let pool = ThreadPool::new(num_threads);
            let (tx, rx) = channel();
            let table = Arc::new(HashTable::new(table_size));

            for i in 0..n_jobs {
                let tx = tx.clone();
                let mv = movelist[i];
                let new_pos = pos.make_move(&mv);
                let table = table.clone();
                pool.execute(move || {
                    let node_count = perft_inner(&new_pos, depth - 1, &table);
                    tx.send(node_count).unwrap();
                    if verbose {
                        println!("{}: {}", mv.to_algebraic(), node_count);
                    }
                })
            }
            rx.iter().take(n_jobs).fold(0, |a, b| a + b)
        };

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

        if let Some(nodes) = table.probe_perft(pos.key, depth) {
            return nodes;
        }

        let mut movelist = UnorderedList::new();
        generate_all(&pos, &mut movelist);

        if depth == 1 {
            return movelist.len() as u64;
        }

        for mv in movelist.iter() {
            let new_pos = pos.make_move(mv);
            nodes += perft_inner(&new_pos, depth - 1, table);
        }

        table.write_perft(pos.key, depth, nodes);
        return nodes;
    }

    pub fn run_perft_suite(num_threads: usize, table_size: usize) {
        let positions = [STARTPOS, TPOS2, TPOS3, TPOS4, TPOS5, TPOS6];
        let depths = [6, 5, 7, 5, 5, 5];
        let mut results = Vec::new();
        for (i, (pos_fen, depth)) in std::iter::zip(positions, depths).enumerate() {
            let pos = Position::from_fen(pos_fen).unwrap();
            let (nodes, duration, nodes_per_second) =
                perft(&pos, depth, num_threads, table_size, false);
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

#[cfg(test)]
mod perft_tests {
    use super::*;
    use engine::DEF_TABLE_SIZE_BYTES;
    use perft::perft;
    use test_case::test_case;

    /// Standard test suite
    #[test_case(STARTPOS, vec![20, 400, 8902, 197281, 4865609, 119060324], 6; "startpos")]
    #[test_case(TPOS2, vec![48, 2039, 97862, 4085603, 193690690], 5; "testpos2")]
    #[test_case(TPOS3, vec![14, 191, 2812, 43238, 674624, 11030083, 178633661], 7; "testpos3")]
    #[test_case(TPOS4, vec![6, 264, 9467, 422333, 15833292], 5; "testpos4")]
    #[test_case(TPOS5, vec![44, 1486, 62379, 2103487, 89941194], 5; "testpos5")]
    #[test_case(TPOS6, vec![46, 2079, 89890, 3894594, 164075551], 5; "testpos6")]
    fn perft_suite(fen: &str, expected_nodes: Vec<u64>, depth: u8) {
        let node = Position::from_fen(fen).unwrap();
        for (exp_node_count, depth) in std::iter::zip(expected_nodes, 1..=depth) {
            let node_count = perft(&node, depth, num_cpus::get(), DEF_TABLE_SIZE_BYTES, false).0;
            assert_eq!(exp_node_count, node_count, "depth {}", depth)
        }
    }

    /// Test suite for testing a variety of niche rules and mechanics.
    #[test_case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888; "illegal ep move #1")]
    #[test_case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133; "illegal ep move #2")]
    #[test_case("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467; "ep capture checks opponent")]
    #[test_case("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072; "short castling gives check")]
    #[test_case("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711; "long castling gives check")]
    #[test_case("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206; "castle rights")]
    #[test_case("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476; "castling prevented")]
    #[test_case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001; "promote out of check")]
    #[test_case("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658; "discovered check")]
    #[test_case("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342; "promote to give check")]
    #[test_case("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683; "under promote to give check")]
    #[test_case("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217; "self statemate")]
    #[test_case("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584; "stalemate & checkmate")]
    #[test_case("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527; "stalemate & checkmate #2")]
    fn talk_chess_perft_tests(fen: &str, depth: u8, expected_nodes: u64) {
        let node = Position::from_fen(fen).unwrap();
        assert_eq!(
            perft(&node, depth, num_cpus::get(), DEF_TABLE_SIZE_BYTES, false).0,
            expected_nodes
        );
    }

    /// Intensive perft tests. Keep ignore flag to prevent from being
    /// run in a normal test suite.
    #[ignore]
    #[test_case(STARTPOS, 3195901860, 7; "startpos")]
    #[test_case(TPOS2, 8031647685, 6; "testpos2")]
    #[test_case(TPOS3, 3009794393, 8; "testpos3")]
    #[test_case(TPOS4, 706045033, 6; "testpos4")]
    #[test_case(TPOS6, 6923051137, 6; "testpos5")]
    fn deep_perft_suite(fen: &str, expected_nodes: u64, depth: u8) {
        let node = Position::from_fen(fen).unwrap();
        let result = perft(&node, depth, num_cpus::get(), DEF_TABLE_SIZE_BYTES, false).0;
        assert_eq!(result, expected_nodes)
    }
}
