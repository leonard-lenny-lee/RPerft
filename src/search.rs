use super::*;
use evaluate::evaluate;
use hash::{HashTable, Probe};
use movegen::{generate, generate_all};
use movelist::{Move, MoveList, OrderedList, UnorderedList};
use position::Position;
use types::NodeType;

const INFINITE: i16 = 30000;
const MAX_DEPTH: u8 = 30;

pub fn search(pos: &Position, depth: u8, table: &mut HashTable) {
    table.age += 1;

    // Use iterative deepening framework
    for depth in 1..=depth {
        search_iteration(pos, depth, table)
    }

    // Best move
    if let Probe::Read(e) = table.probe_search(pos.key, depth) {
        println!(
            "{}",
            v_uci::UciMessage::BestMove {
                best_move: e.best_move.to_uci(),
                ponder: None
            }
        );
    }
}

/// Execute a search iteration within the iterative deepening framework
fn search_iteration(pos: &Position, depth: u8, table: &HashTable) {
    use v_uci::UciInfoAttribute;

    let mut info = SearchInfo::new();
    // Execute search
    alpha_beta(pos, depth, -INFINITE, INFINITE, table, &mut info);

    // Report search results
    let mut uci_info = Vec::new();

    if let Probe::Read(entry) = table.probe_search(pos.key, depth) {
        // PV
        let pv = find_pv(pos, depth, table);
        let mut pv_len = pv.len() as i8;
        uci_info.push(UciInfoAttribute::Pv(pv));

        // Score
        let mate = if entry.score.abs() >= INFINITE - MAX_DEPTH as i16 {
            if entry.score < 0 {
                pv_len *= -1
            }
            Some(pv_len)
        } else {
            None
        };

        uci_info.push(UciInfoAttribute::Score {
            cp: Some(entry.score as i32),
            mate,
            lower_bound: None,
            upper_bound: None,
        });

        // Nodes and time
        uci_info.push(UciInfoAttribute::Nodes(info.nodes));

        uci_info.push(UciInfoAttribute::Time(v_uci::Duration::nanoseconds(
            info.duration_ns(),
        )));

        uci_info.push(UciInfoAttribute::Nps(info.nps()));
    };
    println!("{}", v_uci::UciMessage::Info(uci_info))
}

/// Probe the transposition table for the principal variation line
fn find_pv(pos: &Position, depth: u8, table: &HashTable) -> Vec<v_uci::UciMove> {
    let mut pos = *pos;
    let mut pv = Vec::new();

    for depth in (1..=depth).rev() {
        if let Probe::Read(entry) = table.probe_search(pos.key, depth) {
            if !entry.best_move.is_null() {
                pos = pos.make_move(&entry.best_move);
                pv.push(entry.best_move.to_uci());
            } else {
                break;
            }
        } else {
            break;
        }
    }

    return pv;
}

/// Search a position for the best evaluation using the exhaustative depth
/// first negamax algorithm. Not to be used in release; use as a testing tool
/// to ensure the same results are reached by alpha beta pruning
fn _nega_max(pos: &Position, depth: u8, table: &HashTable) -> i16 {
    let probe = table.probe_search(pos.key, depth);

    if let Probe::Read(entry) = probe {
        return entry.score;
    }

    if depth == 0 {
        return evaluate(pos);
    }

    let mut moves = UnorderedList::new();
    generate_all(pos, &mut moves);

    if moves.len() == 0 {
        let n_checkers = pos.checkers().pop_count();
        if n_checkers > 0 {
            return -INFINITE; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }

    let mut best_move = Move::null();
    let mut best_score = -INFINITE;

    for mv in moves.iter() {
        let new_pos = pos.make_move(mv);
        let evaluation = -_nega_max(&new_pos, depth - 1, table);
        if evaluation > best_score {
            best_score = evaluation;
            best_move = *mv;
        }
    }

    if let Probe::Write = probe {
        table.write_search(pos.key, depth, best_move, best_score, NodeType::PV);
    }

    return best_score;
}

/// Implementation of alpha-beta pruning to search for the best score
pub fn alpha_beta(
    pos: &Position,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    table: &HashTable,
    info: &mut SearchInfo,
) -> i16 {
    debug_assert!(beta > alpha);

    if depth == 0 {
        return quiescence(pos, alpha, beta, info);
    }

    info.nodes += 1;

    if pos.ply >= MAX_DEPTH {
        return evaluate(pos);
    }

    let probe = table.probe_search(pos.key, depth);

    let tt_move = match probe {
        Probe::Read(e) => return e.score,
        Probe::ReadWrite(ref e) => e.best_move,
        _ => Move::null(),
    };

    let write = matches!(probe, Probe::Write | Probe::ReadWrite(_));

    let mut moves = OrderedList::new();
    generate_all(pos, &mut moves);

    if moves.len() == 0 {
        let n_checkers = pos.checkers().pop_count();
        if n_checkers > 0 {
            return -INFINITE + pos.ply as i16; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }

    // Complete move ordering
    moves.score(tt_move, info.killer_table.get(pos.ply), &info.history_table);
    moves.sort();

    let mut best_score = -INFINITE;
    let mut best_move = Move::null();
    let old_alpha = alpha;

    for (mv, _) in moves.0.iter() {
        let new_pos = pos.make_move(mv);
        let score = -alpha_beta(&new_pos, depth - 1, -beta, -alpha, table, info);

        if score > best_score {
            best_score = score;
            best_move = *mv;

            if score > alpha {
                if score >= beta {
                    // Beta cutoff; "cut" node
                    if write {
                        table.write_search(pos.key, depth, best_move, beta, NodeType::Cut);
                    }
                    // Record "killer" moves for move ordering
                    // These are quiet moves which trigger a beta cutoff
                    if !mv.is_capture() {
                        info.killer_table.set(pos.ply, mv)
                    }
                    return beta;
                }
                // Record the history heuristic for quiet moves
                if !mv.is_capture() {
                    info.history_table.set(mv.from(), mv.to(), depth);
                }
                alpha = score;
            }
        }
    }

    debug_assert!(alpha >= old_alpha);

    if write {
        if alpha != old_alpha {
            table.write_search(pos.key, depth, best_move, best_score, NodeType::All)
        } else {
            table.write_search(pos.key, depth, best_move, alpha, NodeType::PV);
        }
    }

    return alpha;
}

fn quiescence(pos: &Position, mut alpha: i16, beta: i16, info: &mut SearchInfo) -> i16 {
    info.nodes += 1;

    if pos.ply > MAX_DEPTH as u8 {
        return evaluate(pos);
    }

    // "Stand pat" decision
    let score = evaluate(pos);

    if score >= beta {
        return beta;
    }

    if score > alpha {
        alpha = score;
    }

    let mut moves = OrderedList::new();
    let checkers = pos.checkers();
    let our_attacks = pos.attack_sq(); // All squares our pieces are attacking
    let captures = our_attacks & pos.them.all;

    // If in check, the priority is to resolve the check
    if checkers.is_not_empty() {
        generate(types::GenType::Evasions(checkers), pos, &mut moves);

        if moves.len() == 0 {
            return -INFINITE;
        }
    }
    // Enumerate the through the captures only
    else if captures != EMPTY_BB {
        generate(types::GenType::Captures, pos, &mut moves);
        moves.sort();
    }
    // No captures and not in check so stop quiescing
    else {
        return alpha;
    };

    for (mv, _) in moves.0.iter() {
        let new_pos = pos.make_move(mv);
        let score = -quiescence(&new_pos, -beta, -alpha, info);

        if score > alpha {
            if score >= beta {
                return beta;
            }
            alpha = score
        }
    }

    return alpha;
}

pub struct SearchInfo {
    nodes: u64,
    killer_table: KillerTable,
    history_table: HistoryTable,
    start: std::time::Instant,
}

impl SearchInfo {
    fn new() -> Self {
        Self {
            nodes: 0,
            killer_table: KillerTable::new(),
            history_table: HistoryTable::new(),
            start: std::time::Instant::now(),
        }
    }

    fn duration_ns(&self) -> i64 {
        std::cmp::max(self.start.elapsed().as_nanos() as i64, 1)
    }

    fn nps(&self) -> u64 {
        ((self.nodes as f64 / self.duration_ns() as f64) * 1_000_000_000.0).round() as u64
    }
}

struct KillerTable([[Move; 2]; MAX_DEPTH as usize]);

impl KillerTable {
    fn new() -> Self {
        Self([[Move::null(); 2]; MAX_DEPTH as usize])
    }

    fn get(&self, ply: u8) -> [Move; 2] {
        return self.0[ply as usize];
    }

    fn set(&mut self, ply: u8, mv: &Move) {
        let moves = &mut self.0[ply as usize];
        (moves[0], moves[1]) = (*mv, moves[0])
    }
}

pub struct HistoryTable([[u16; 64]; 64]);

impl HistoryTable {
    fn new() -> Self {
        Self([[0; 64]; 64])
    }

    pub fn get(&self, from: BB, to: BB) -> u16 {
        self.0[from.to_sq()][to.to_sq()]
    }

    fn set(&mut self, from: BB, to: BB, depth: u8) {
        self.0[from.to_sq()][to.to_sq()] += depth as u16;
    }
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
mod tests {
    use super::*;
    use engine::DEF_TABLE_SIZE_BYTES;
    use test_case::test_case;

    // Test the equivalency of search result from negamax and alpha beta
    #[ignore]
    #[test_case(STARTPOS, 2; "startpos")]
    #[test_case(TPOS2, 2; "testpos2")]
    #[test_case(TPOS3, 2; "testpos3")]
    #[test_case(TPOS4, 2; "testpos4")]
    #[test_case(TPOS5, 2; "testpos5")]
    #[test_case(TPOS6, 2; "testpos6")]
    fn test_alpha_beta(fen: &str, depth: u8) {
        let pos = Position::from_fen(fen).unwrap();
        let mut table = HashTable::new(DEF_TABLE_SIZE_BYTES);
        let mut info = SearchInfo::new();

        let alpha_beta = alpha_beta(&pos, depth, -INFINITE, INFINITE, &table, &mut info);
        table.clear();

        let negamax = _nega_max(&pos, depth, &table);
        assert_eq!(alpha_beta, negamax)
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
