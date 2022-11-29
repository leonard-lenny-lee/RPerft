/// Tests to guarantee move generation fidelity and benchmarking 

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
        // if pos.data.w_pieces.rook & 1 << 56 != EMPTY_BB {
        //     println!("{}{}", bittools::bitmask_to_algebraic(mv.src), bittools::bitmask_to_algebraic(mv.target))
        // }
        let new_pos = apply_move(pos, &mv);
        nodes += perft(&new_pos, depth-1, maps);
    }
    return nodes
}

/// Tests to compare the number of nodes generated in these standard perft
/// positions against the consensus. Note these may be slow to run and so
/// would recommend running in release mode cargo test --release
#[test_case(DEFAULT_FEN, vec![20, 400, 8902, 197281, 4865609, 119060324], 6; "starting_position")]
#[test_case(POSITION_2, vec![48, 2039, 97862, 4085603, 193690690], 5; "position_two")]
#[test_case(POSITION_3, vec![14, 191, 2812, 43238, 674624, 11030083, 178633661], 7; "position_three")]
#[test_case(POSITION_4, vec![6, 264, 9467, 422333, 15833292], 5; "position_four")]
#[test_case(POSITION_5, vec![44, 1486, 62379, 2103487, 89941194], 5; "position_five")]
#[test_case(POSITION_6, vec![46, 2079, 89890, 3894594, 164075551], 5; "position_six")]
fn perft_test(fen: &str, expected_nodes: Vec<i32>, depth: i8) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    for dpt in 1..depth + 1 {
        let result = perft(&pos, dpt, &maps);
        assert_eq!(expected_nodes[dpt as usize - 1], result, "depth {}", dpt)
    }
}

#[test_case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888; "illegal ep move #1")]
#[test_case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133; "illegal ep move #2")]
#[test_case("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467; "ep capture checks opponent")]
#[test_case("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072; "short castling gives check")]
#[test_case("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711; "long castling gives check")]
#[test_case("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206; "castle rights")]
#[test_case("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476; "castling prevented")]
#[test_case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001; "promote out of check")]
#[test_case("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658; "discovered check")]
#[test_case("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342; "promote to give check")]
#[test_case("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683; "under promote to give check")]
#[test_case("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217; "self statemate")]
#[test_case("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584; "stalemate & checkmate")]
#[test_case("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527; "stalemate & checkmate #2")]
fn talk_chess_perft_tests(fen: &str, depth: i8, expected_nodes: i32) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    assert_eq!(perft(&pos, depth, &maps), expected_nodes);
}

// Not part of perft test suite, useful for debugging.
#[ignore]
#[test]
fn perft_debug() {
    let fen = "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1";
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    perft_divide(&pos, 4, &maps);
}