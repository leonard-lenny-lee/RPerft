use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::engine::*;
use position::Position;
use common::*;
use global::maps::Maps;

fn setup() -> Position {
    Position::new_from_fen(POSITION_2.to_string())
}

pub fn find_moves_benchmark(c: &mut Criterion) {
    use search::move_generation::*;
    use position::analysis_tools::*;

    let pos = &setup();
    let maps = &Maps::new();
    let mut move_vec = Vec::new();

    c.bench_function(
        "find_moves",
        |b| b.iter(
            || find_moves(
                black_box(pos), black_box(maps)
            )
        )
    );

    c.bench_function(
        "find_unsafe_squares",
        |b| b.iter(
            || find_unsafe_squares(
                black_box(pos), black_box(maps)
            )
        )
    );

    c.bench_function(
        "find_checkers",
        |b| b.iter(
            || find_checkers(
                black_box(pos), black_box(maps)
            )
        )
    );

    c.bench_function(
        "get_pinned_pieces_for",
        |b| b.iter(
            || get_pinned_pieces_for(
                black_box(pos), black_box(maps)
            )
        )
    );

    c.bench_function(
        "find_pawn_moves",
        |b| b.iter(
            || find_pawn_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                black_box(PawnMove::DoublePush),
                FILLED_BB,
                FILLED_BB,
                EMPTY_BB,
            )
        )
    );

    c.bench_function(
        "find_knight_moves",
        |b| b.iter(
            || find_knight_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                &Maps::new(),
                FILLED_BB,
                FILLED_BB,
                EMPTY_BB,
            )
        )
    );

}

criterion_group!(benches, find_moves_benchmark);
criterion_main!(benches);