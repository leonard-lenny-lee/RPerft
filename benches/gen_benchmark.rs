use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::engine::*;
use position::Position;
use common::*;

fn setup() -> Position {
    Position::new_from_fen(POSITION_2.to_string())
}

pub fn find_moves_benchmark(c: &mut Criterion) {
    use search::move_generation::*;
    use position::analysis_tools::*;

    let pos = &setup();
    let mut move_vec = Vec::new();

    c.bench_function(
        "find_moves",
        |b| b.iter(
            || find_moves(
                black_box(pos)
            )
        )
    );

    c.bench_function(
        "find_unsafe_squares",
        |b| b.iter(
            || find_unsafe_squares(
                black_box(pos)
            )
        )
    );

    c.bench_function(
        "find_checkers",
        |b| b.iter(
            || find_checkers(
                black_box(pos)
            )
        )
    );

    c.bench_function(
        "get_pinned_pieces_for",
        |b| b.iter(
            || get_pinned_pieces_for(
                black_box(pos)
            )
        )
    );

    c.bench_function(
        "find_pawn_moves",
        |b| b.iter(
            || find_single_pushes(
                black_box(&mut move_vec),
                black_box(&pos),
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
                FILLED_BB,
                FILLED_BB,
                EMPTY_BB,
            )
        )
    );

    c.bench_function(
        "find_king_moves",
        |b| b.iter(
            || find_king_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                EMPTY_BB
            )
        )
    );

    c.bench_function(
        "find_sliding_moves",
        |b| b.iter(
            || find_sliding_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                SlidingPiece::Bishop,
                FILLED_BB,
                FILLED_BB,
                EMPTY_BB,
            )
        )
    );

    c.bench_function(
        "find_en_passant",
        |b| b.iter(
            || find_en_passant_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                FILLED_BB,
                FILLED_BB,
                EMPTY_BB
            )
        )
    );

    c.bench_function(
        "find_castling",
        |b| b.iter(
            || find_castling_moves(
                black_box(&mut move_vec),
                black_box(&pos),
                EMPTY_BB,
            )
        )
    );

}

pub fn apply_move_benchmark(c: &mut Criterion) {
    use search::move_generation::*;
    use search::apply_move::apply_move;

    let pos = &setup();
    let move_vec = find_moves(pos);

    c.bench_function(
        "apply_move",
        |b| b.iter(
            || apply_move(
                pos,
                &move_vec[0]
            )
        )
    );

}

criterion_group!(benches, find_moves_benchmark, apply_move_benchmark);
criterion_main!(benches);