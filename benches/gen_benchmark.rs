use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::engine::{*, search::{SearchNode, MoveList}};
use position::Position;
use common::*;

fn setup() -> Position {
    Position::new_from_fen(POSITION_2.to_string())
}

fn setup_node() -> SearchNode {
    SearchNode::new_from_fen(POSITION_2.to_string())
}

pub fn find_moves_benchmark(c: &mut Criterion) {

    let pos = setup();
    let mut move_list = MoveList::new();

    c.bench_function(
        "find_moves",
        |b| b.iter(
            || pos.find_moves()
        )
    );

    c.bench_function(
        "find_unsafe_squares",
        |b| b.iter(
            || pos.unsafe_squares()
        )
    );

    c.bench_function(
        "find_checkers",
        |b| b.iter(
            || pos.find_checkers()
        )
    );

    c.bench_function(
        "get_pinned_pieces_for",
        |b| b.iter(
            || pos.pinned_pieces()
        )
    );

}

pub fn apply_move_benchmark(c: &mut Criterion) {
    use search::apply_move::apply_move;

    let node = &setup_node();
    let move_vec = node.pos.find_moves();

    c.bench_function(
        "apply_move",
        |b| b.iter(
            || apply_move(
                node,
                &move_vec[0]
            )
        )
    );

}

criterion_group!(benches, find_moves_benchmark, apply_move_benchmark);
criterion_main!(benches);