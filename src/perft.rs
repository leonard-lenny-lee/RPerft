// Search algorithm
use super::*;

use cache::Cache;
use movegen::generate_all;
use position::Position;

pub fn perft(
    pos: &Position,
    depth: u8,
    num_threads: usize,
    cache_size: usize,
    verbose: bool,
) -> (u64, f64, f64) {
    let start = std::time::Instant::now();

    let mut movelist = movelist::MoveVec::new();
    generate_all(&pos, &mut movelist);

    let mut nodes = 0;
    if depth == 0 {
        nodes = 1
    } else if depth == 1 {
        nodes = movelist.len() as u64
    } else {
        if num_threads > 1 {
            let n_jobs = movelist.len();
            let pool = threadpool::ThreadPool::new(num_threads);
            let (tx, rx) = std::sync::mpsc::channel();
            let table = std::sync::Arc::new(Cache::new(cache_size));

            for i in 0..n_jobs {
                let tx = tx.clone();
                let mv = movelist[i];
                let new_position = pos.make_move(&mv);
                let table = table.clone();
                pool.execute(move || {
                    let node_count = perft_inner(&new_position, depth - 1, &table);
                    tx.send(node_count).unwrap();
                    if verbose {
                        println!("{}: {}", mv.to_algebraic(), node_count);
                    }
                })
            }
            nodes = rx.iter().take(n_jobs).fold(0, |a, b| a + b);
        } else {
            let table = std::sync::Arc::new(Cache::new(cache_size));
            for mv in movelist.iter() {
                let new_position = pos.make_move(mv);
                nodes += perft_inner(&new_position, depth - 1, &table)
            }
        }
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

fn perft_inner(position: &Position, depth: u8, table: &std::sync::Arc<Cache>) -> u64 {
    if let Some(nodes) = table.fetch(position.key, depth) {
        return nodes;
    }
    if depth == 1 {
        let mut movelist = movelist::MoveCounter::default();
        generate_all(position, &mut movelist);
        return movelist.count as u64;
    }
    let mut movelist = movelist::MoveVec::new();
    generate_all(&position, &mut movelist);
    let mut nodes = 0;
    for mv in movelist.iter() {
        let new_position = position.make_move(mv);
        nodes += perft_inner(&new_position, depth - 1, table);
    }
    table.store(position.key, depth, nodes);
    return nodes;
}

pub fn run_perft_benchmark_suite(num_threads: usize, table_size: usize) {
    use constants::fen::*;

    let positions = [START, TEST_2, TEST_3, TEST_4, TEST_5, TEST_6];
    let depths = [6, 5, 7, 5, 5, 5];
    let mut results = Vec::new();
    for (i, (pos_fen, depth)) in std::iter::zip(positions, depths).enumerate() {
        let pos = Position::from_fen(pos_fen).unwrap();
        let (nodes, duration, nodes_per_second) =
            perft(&pos, depth, num_threads, table_size, false);
        results.push((i + 1, nodes, duration, nodes_per_second));
    }
    // Report results
    println!("+{}+", "-".repeat(35));
    println!(
        "|{:>3} |{:>11} |{:>6} |{:>8} |",
        "#", "Nodes", "sec", "MN/s"
    );
    println!("+{}+", "-".repeat(35));
    for (n, nodes, duration, nodes_per_second) in results {
        println!(
            "|{:>3} |{:>11} |{:>6} |{:>8} |",
            n,
            nodes,
            format!("{:.2}", duration),
            format!("{:.2}", nodes_per_second)
        )
    }
    println!("+{}+", "-".repeat(35))
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::fen::*;
    use constants::DEFAULT_TABLE_SIZE_BYTES;
    use test_case::test_case;

    /// Standard test suite
    #[test_case(START, vec![20, 400, 8902, 197281, 4865609, 119060324], 6; "startpos")]
    #[test_case(TEST_2, vec![48, 2039, 97862, 4085603, 193690690], 5; "testpos2")]
    #[test_case(TEST_3, vec![14, 191, 2812, 43238, 674624, 11030083, 178633661], 7; "testpos3")]
    #[test_case(TEST_4, vec![6, 264, 9467, 422333, 15833292], 5; "testpos4")]
    #[test_case(TEST_5, vec![44, 1486, 62379, 2103487, 89941194], 5; "testpos5")]
    #[test_case(TEST_6, vec![46, 2079, 89890, 3894594, 164075551], 5; "testpos6")]
    fn perft_suite(fen: &str, expected_nodes: Vec<u64>, depth: u8) {
        let node = Position::from_fen(fen).unwrap();
        for (exp_node_count, depth) in std::iter::zip(expected_nodes, 1..=depth) {
            let node_count = perft(
                &node,
                depth,
                num_cpus::get(),
                DEFAULT_TABLE_SIZE_BYTES,
                false,
            )
            .0;
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
            perft(
                &node,
                depth,
                num_cpus::get(),
                DEFAULT_TABLE_SIZE_BYTES,
                false
            )
            .0,
            expected_nodes
        );
    }

    /// Intensive perft tests. Keep ignore flag to prevent from being
    /// run in a normal test suite.
    #[ignore]
    #[test_case(START, 3195901860, 7; "startpos")]
    #[test_case(TEST_2, 8031647685, 6; "testpos2")]
    #[test_case(TEST_3, 3009794393, 8; "testpos3")]
    #[test_case(TEST_4, 706045033, 6; "testpos4")]
    #[test_case(TEST_6, 6923051137, 6; "testpos5")]
    fn deep_perft_suite(fen: &str, expected_nodes: u64, depth: u8) {
        let node = Position::from_fen(fen).unwrap();
        let result = perft(
            &node,
            depth,
            num_cpus::get(),
            DEFAULT_TABLE_SIZE_BYTES,
            false,
        )
        .0;
        assert_eq!(result, expected_nodes)
    }
}
