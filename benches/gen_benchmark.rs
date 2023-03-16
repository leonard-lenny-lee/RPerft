use chess::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use movegen::find_moves;
use position::Position;

fn setup() -> Position {
    const TPOS2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    Position::from_fen(TPOS2).unwrap()
}

pub fn find_moves_benchmark(c: &mut Criterion) {
    let pos = setup();

    c.bench_function("find_moves", |b| b.iter(|| black_box(find_moves(&pos))));

    c.bench_function("find_unsafe_squares", |b| {
        b.iter(|| black_box(pos.unsafe_squares()))
    });

    c.bench_function("find_checkers", |b| {
        b.iter(|| black_box(pos.find_checkers()))
    });

    c.bench_function("get_pinned_pieces_for", |b| {
        b.iter(|| black_box(pos.pinned_pieces()))
    });
}

pub fn apply_move_benchmark(c: &mut Criterion) {
    let pos = setup();
    let move_vec = find_moves(&pos);

    c.bench_function("apply_move", |b| b.iter(|| pos.do_move(&move_vec[0])));
}

criterion_group!(benches, find_moves_benchmark, apply_move_benchmark);
criterion_main!(benches);
