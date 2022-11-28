use chess_engine::*;
use search::move_generation::{find_moves, apply_move::apply_move};
use position::Position;
use common::*;
use global::maps::Maps;
use test_case::test_case;

/// Provides the number of nodes for down each branch of the first depth layer
/// search. Useful for perft debugging purposes
fn perft_divide(pos: &Position, depth: i8, maps: &Maps) {
    let mut nodes = 0;
    let moves = find_moves(pos, maps);
    for mv in moves {
        let new_pos = apply_move(pos, &mv);
        let branch_nodes = perft(&new_pos, depth-1, maps);
        // Report branch
        let src = bittools::bitmask_to_algebraic(mv.src);
        let target = bittools::bitmask_to_algebraic(mv.target);
        let promotion_piece;
        match mv.promotion_piece {
            Promotion::None => promotion_piece = "",
            Promotion::Rook => promotion_piece = "r",
            Promotion::Knight => promotion_piece = "n",
            Promotion::Bishop => promotion_piece = "b",
            Promotion::Queen => promotion_piece = "q"
        }
        println!("{}{}{}: {}", src, target, promotion_piece, branch_nodes);
        nodes += branch_nodes;
    }
    println!("\nnodes: {}", nodes)
}

fn perft(pos: &Position, depth: i8, maps: &Maps) -> i32 {
    let mut nodes = 0;
    if depth == 0 {
        return 1;
    }
    let moves = find_moves(pos, maps);
    for mv in moves {
        // if pos.data.w_pieces.rook == 1<<9 {
        //     println!("{}{}", bittools::bitmask_to_algebraic(mv.src), bittools::bitmask_to_algebraic(mv.target))
        // }
        let new_pos = apply_move(pos, &mv);
        nodes += perft(&new_pos, depth-1, maps);
    }
    return nodes
}

#[test_case(DEFAULT_FEN, vec![20, 400, 8902, 197281, 4865609], 5; "starting_position")]
#[test_case(POSITION_2, vec![48, 2039, 97862, 4085603], 4; "position_two")]
#[test_case(POSITION_3, vec![14, 191, 2812, 43238, 674624, 11030083], 6; "position_three")]
#[test_case(POSITION_4, vec![6, 264, 9467, 422333, 15833292], 5; "position_four")]
#[test_case(POSITION_5, vec![44, 1486, 62379, 2103487], 4; "position_five")]
#[test_case(POSITION_6, vec![46, 2079, 89890, 3894594], 4; "position_six")]
fn perft_test(fen: &str, expected_nodes: Vec<i32>, depth: i8) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    for dpt in 1..depth + 1 {
        let result = perft(&pos, dpt, &maps);
        assert_eq!(expected_nodes[dpt as usize - 1], result, "depth {}", dpt)
    }
}

// Not part of perft test suite, useful for debugging.
#[ignore]
#[test]
fn perft_debug() {
    let fen = POSITION_6;
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    perft_divide(&pos, 4, &maps);
}