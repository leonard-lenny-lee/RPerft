use chess::{movegen::generate_all, *};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use movelist::UnorderedList;
use position::Position;

fn setup() -> (Position, UnorderedList) {
    const TPOS2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    (Position::from_fen(TPOS2).unwrap(), UnorderedList::new())
}

pub fn find_moves_benchmark(c: &mut Criterion) {
    let (pos, mut movelist) = setup();

    c.bench_function("find_moves", |b| {
        b.iter(|| black_box(movegen::generate_all(&pos, &mut movelist)))
    });

    c.bench_function("find_unsafe_squares", |b| {
        b.iter(|| black_box(pos.unsafe_sq()))
    });

    c.bench_function("find_checkers", |b| b.iter(|| black_box(pos.checkers())));

    c.bench_function("get_pinned_pieces_for", |b| {
        b.iter(|| black_box(pos.pinned()))
    });
}

pub fn apply_move_benchmark(c: &mut Criterion) {
    let (mut pos, mut movelist) = setup();
    generate_all(&pos, &mut movelist);

    c.bench_function("apply_move", |b| b.iter(|| pos.make_move(&movelist[0])));
}

criterion_group!(benches, find_moves_benchmark, apply_move_benchmark);
criterion_main!(benches);
