// Search algorithm
use super::*;

use std::cmp::Ordering;
use std::iter::zip;
use std::sync::{mpsc::channel, Arc};

use threadpool::ThreadPool;

use cache::*;
use movegen::generate_all;
use movelist::*;
use position::Position;
use stats::Stats;

mod cfg;
mod stats;

#[cfg(test)]
mod tests;

pub fn perft_wrapper(
    fen: &str,
    depth: u8,
    cache_size: usize,
    multithreading: bool,
    detailed: bool,
) {
    let mut table = prettytable::Table::new();

    table.add_row(Stats::start_row(detailed));

    let pos = match Position::from_fen(fen) {
        Ok(p) => p,
        Err(_) => {
            log::error!("Invalid FEN: {fen}");
            return;
        }
    };

    println!("{pos}");

    let cfg = cfg::Config::new(multithreading, cache_size, detailed);
    cfg.report().printstd();

    for d in 1..=depth {
        let stats = if cfg.detailed {
            perft::<Entry4xU64>(&pos, d, &cfg)
        } else {
            perft::<Entry2xU64>(&pos, d, &cfg)
        };
        table.add_row(stats.to_row(detailed));
    }

    println!();
    table.printstd();
}

pub fn run_perft_benchmark_suite(
    cache_size: usize,
    multithreading: bool,
    deep: bool,
    detailed: bool,
) {
    use constants::fen::*;

    let cfg = cfg::Config::new(multithreading, cache_size, detailed);

    let tests = [STARTING_FEN, TEST_2, TEST_3, TEST_4, TEST_5, TEST_6];
    let depths;
    if deep {
        depths = [7, 6, 8, 6, 6, 6]
    } else {
        depths = [6, 5, 7, 5, 5, 5]
    }

    let mut table = prettytable::Table::new();

    let mut start_row = Stats::start_row(detailed);
    start_row.insert_cell(0, cell!("Bench #"));
    table.add_row(start_row);

    for (i, (fen, depth)) in zip(tests, depths).enumerate() {
        let pos = Position::from_fen(fen).expect("valid fen");
        let stats = if cfg.detailed {
            perft::<Entry4xU64>(&pos, depth, &cfg)
        } else {
            perft::<Entry2xU64>(&pos, depth, &cfg)
        };
        let mut row = stats.to_row(detailed);
        row.insert_cell(0, cell!(i));
        table.add_row(row);
    }

    cfg.report().printstd();
    println!();
    table.printstd();
}

fn perft<T: SizedEntry + 'static>(pos: &Position, depth: u8, cfg: &cfg::Config) -> Stats {
    let caching = cfg.cache_size > 0;
    let num_threads;

    if cfg.multithreading && depth > 3 {
        num_threads = cfg.num_threads
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
            let cache = Arc::new(Cache::<T>::new(cfg.cache_size));

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
    let mut count = MoveCounter::default();
    for mv in movelist.iter() {
        let new_pos = pos.make_move(mv);
        count += perft_inner(&new_pos, depth - 1);
    }
    return count;
}

fn perft_inner_cache<T: SizedEntry>(
    pos: &Position,
    depth: u8,
    cache: &Arc<Cache<T>>,
) -> MoveCounter {
    if let Some(count) = cache.read(pos.key, depth) {
        return count;
    }
    if depth == 1 {
        let mut count = MoveCounter::default();
        generate_all(pos, &mut count);
        return count;
    }
    let mut moves = MoveVec::new();
    generate_all(&pos, &mut moves);
    let mut count = MoveCounter::default();
    for mv in moves.iter() {
        let new_position = pos.make_move(mv);
        count += perft_inner_cache(&new_position, depth - 1, cache);
    }
    cache.write(pos.key, depth, &count);
    return count;
}
