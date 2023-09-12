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

pub fn perft_wrapper(fen: &str, depth: u8, cache_size: usize, multithreading: bool) {
    let mut table = prettytable::Table::new();

    table.add_row(Stats::start_row());

    let pos = match Position::from_fen(fen) {
        Ok(p) => p,
        Err(_) => {
            log::error!("Invalid FEN: {fen}");
            return;
        }
    };

    for d in 1..=depth {
        let stats = perft(&pos, d, cache_size, multithreading);
        table.add_row(stats.to_row());
    }

    table.printstd();
}

fn perft(pos: &Position, depth: u8, cache_size: usize, multithreading: bool) -> Stats {
    let caching = cache_size > 0;
    let num_threads;

    if multithreading && depth > 3 {
        num_threads = num_cpus::get()
    } else {
        num_threads = 1
    };

    let mut stats = Stats::new(depth);

    match depth.cmp(&1) {
        Ordering::Less => stats.count.nodes += 1,
        Ordering::Equal => generate_all(&pos, &mut stats.count),
        Ordering::Greater => {
            let mut moves = MoveVec::new();
            generate_all(&pos, &mut moves);
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
            stats.count += rx
                .iter()
                .take(n_jobs)
                .fold(MoveCounter::default(), |a, b| a + b)
        }
    };
    stats.end();
    stats
}

fn perft_inner(pos: &Position, depth: u8) -> MoveCounter {
    if depth == 1 {
        let mut movelist = MoveCounter::default();
        generate_all(pos, &mut movelist);
        return movelist;
    }

    let mut movelist = MoveVec::new();
    generate_all(pos, &mut movelist);
    let mut nodes = MoveCounter::default();
    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
        nodes += perft_inner(&new_pos, depth - 1);
    }
    return nodes;
}

fn perft_inner_cache(pos: &Position, depth: u8, cache: &Arc<Cache>) -> MoveCounter {
    if let Some(nodes) = cache.fetch(pos.key, depth) {
        return nodes;
    }
    if depth == 1 {
        let mut movelist = MoveCounter::default();
        generate_all(pos, &mut movelist);
        return movelist;
    }
    let mut movelist = MoveVec::new();
    generate_all(&pos, &mut movelist);
    let mut nodes = MoveCounter::default();
    for mv in movelist.iter() {
        let new_position = pos.make_move(mv);
        nodes += perft_inner_cache(&new_position, depth - 1, cache);
    }
    cache.store(pos.key, depth, &nodes);
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

    let mut table = prettytable::Table::new();

    table.add_row(Stats::start_row());

    for (fen, depth) in zip(tests, depths) {
        let pos = Position::from_fen(fen).expect("valid fen");
        let stats = perft(&pos, depth, cache_size, multithreading);
        table.add_row(stats.to_row());
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

    table.printstd();
}
