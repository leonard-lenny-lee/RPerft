use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::engine::*;
use position::Position;
use common::*;

fn setup() -> Position {
    Position::from_fen(POSITION_2.to_string()).unwrap()
}

pub fn find_moves_benchmark(c: &mut Criterion) {

    let pos = setup();

    c.bench_function(
        "find_moves",
        |b| b.iter(
            || black_box(pos.find_moves())
        )
    );

    c.bench_function(
        "find_unsafe_squares",
        |b| b.iter(
            || black_box(pos.unsafe_squares())
        )
    );

    c.bench_function(
        "find_checkers",
        |b| b.iter(
            || black_box(pos.find_checkers())
        )
    );

    c.bench_function(
        "get_pinned_pieces_for",
        |b| b.iter(
            || black_box(pos.pinned_pieces())
        )
    );

}

pub fn apply_move_benchmark(c: &mut Criterion) {

    let pos = setup();
    let move_vec = pos.find_moves();

    c.bench_function(
        "apply_move",
        |b| b.iter(
            || pos.make_move(
                black_box(&move_vec[0])
            )
        )
    );

}

criterion_group!(benches, find_moves_benchmark, apply_move_benchmark);
criterion_main!(benches);