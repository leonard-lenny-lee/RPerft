// Search algorithm
use super::*;

use std::cmp::Ordering;
use std::iter::zip;
use std::sync::{mpsc::channel, Arc};

use threadpool::ThreadPool;

use cache::Cache;
use movegen::generate_all;
use movelist::*;
use position::Position;
use stats::Stats;

mod stats;

#[cfg(test)]
mod tests;

pub fn perft(fen: &str, depth: u8, cache_size: usize, multithreading: bool) -> Result<Stats, ()> {
    let pos = Position::from_fen(fen)?;

    let caching = cache_size > 0;
    let num_threads = if multithreading { num_cpus::get() } else { 1 };

    let mut stats = stats::Stats::new(fen, depth, cache_size, num_threads);
    let mut moves = MoveVec::new();
    generate_all(&pos, &mut moves);

    let nodes = match depth.cmp(&1) {
        Ordering::Less => 1,
        Ordering::Equal => moves.len() as u64,
        Ordering::Greater => {
            let n_jobs = moves.len();
            let pool = ThreadPool::new(num_threads);
            let (tx, rx) = channel();
            let cache = Arc::new(Cache::new(cache_size));

            for i in 0..n_jobs {
                let tx = tx.clone();
                let mv = moves[i];
                let new_pos = pos.make_move(&mv);
                let cache = cache.clone();
                pool.execute(move || {
                    let node_count = if caching {
                        perft_inner_cache(&new_pos, depth - 1, &cache)
                    } else {
                        perft_inner(&new_pos, depth - 1)
                    };
                    tx.send(node_count).unwrap()
                })
            }
            rx.iter().take(n_jobs).fold(0, |a, b| a + b)
        }
    };
    stats.end(nodes);
    Ok(stats)
}

fn perft_inner(pos: &Position, depth: u8) -> u64 {
    if depth == 1 {
        let mut movelist = MoveCounter::default();
        generate_all(pos, &mut movelist);
        return movelist.count;
    }

    let mut movelist = MoveVec::new();
    generate_all(pos, &mut movelist);
    let mut nodes = 0;
    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
        nodes += perft_inner(&new_pos, depth - 1);
    }
    return nodes;
}

fn perft_inner_cache(pos: &Position, depth: u8, cache: &Arc<Cache>) -> u64 {
    if let Some(nodes) = cache.fetch(pos.key, depth) {
        return nodes;
    }
    if depth == 1 {
        let mut movelist = MoveCounter::default();
        generate_all(pos, &mut movelist);
        return movelist.count;
    }
    let mut movelist = MoveVec::new();
    generate_all(&pos, &mut movelist);
    let mut nodes = 0;
    for mv in movelist.iter() {
        let new_position = pos.make_move(mv);
        nodes += perft_inner_cache(&new_position, depth - 1, cache);
    }
    cache.store(pos.key, depth, nodes);
    return nodes;
}

pub fn run_perft_benchmark_suite(cache_size: usize, multithreading: bool, deep: bool) {
    use constants::fen::*;

    let tests = [STARTING_FEN, TEST_2, TEST_3, TEST_4, TEST_5, TEST_6];
    let depths;
    if deep {
        depths = [7, 6, 8, 6, 6, 6]
    } else {
        depths = [6, 5, 7, 5, 5, 5]
    }
    let mut results = Vec::new();
    for (fen, depth) in zip(tests, depths) {
        results.push(perft(fen, depth, cache_size, multithreading).expect("valid fen"));
    }

    if multithreading {
        println!("Multi-threading ENABLED... {} threads", num_cpus::get())
    } else {
        println!("Multi-threading DISABLED")
    }

    if cache_size > 0 {
        println!(
            "Caching ENABLED... {:.2} Mb",
            cache_size as f64 / 1_000_000.0
        );
    } else {
        println!("Caching DISABLED")
    }

    // Report results
    println!("+{}+", "-".repeat(35));
    println!(
        "|{:>3} |{:>11} |{:>6} |{:>8} |",
        "#", "Nodes", "sec", "MN/s"
    );
    println!("+{}+", "-".repeat(35));
    for (n, stats) in results.iter().enumerate() {
        println!(
            "|{:>3} |{:>11} |{:>6} |{:>8} |",
            n,
            stats.node_count,
            format!("{:.2}", stats.duration_sec),
            format!("{:.2}", stats.m_nodes_per_sec)
        )
    }
    println!("+{}+", "-".repeat(35))
}
