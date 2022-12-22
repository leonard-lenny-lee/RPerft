/// Tests to guarantee move enumeration fidelity and benchmarking 

use chess_engine::engine::*;
use transposition::TranspositionTable;
use search::apply_move::apply_move;
use search::SearchNode;
use common::*;
use test_case::test_case;

const HASHING_ENABLED: bool = false;

/// Provides the number of nodes for down each branch of the first depth layer
/// search. Useful for perft debugging purposes
fn perft_divided(root_node: &SearchNode, depth: i8) -> i64 {
    assert!(depth >= 1);
    let mut nodes = 0;
    let move_list = root_node.pos.find_moves();
    for mv in move_list.iter() {
        let new_node = apply_move(root_node, mv);
        let branch_nodes;
        if depth == 1 {
            branch_nodes = 1
        } else {
            branch_nodes = perft_inner(&new_node, depth-1)
        }
        // Report branch
        let src = mv.src().to_algebraic();
        let target = mv.target().to_algebraic();
        let mut promotion_piece = "";
        if mv.is_promotion() {
            match mv.promotion_piece() {
                2 => promotion_piece = "r",
                3 => promotion_piece = "n",
                4 => promotion_piece = "b",
                5 => promotion_piece = "q",
                _ => ()
            }
        }
        println!("{}{}{}: {}", src, target, promotion_piece, branch_nodes);
        nodes += branch_nodes;
    }
    println!("\nnodes: {}", nodes);
    return nodes
}

fn perft(root_node: &SearchNode, depth: i8) -> i64 {
    assert!(depth >= 1);
    if HASHING_ENABLED {
        let mut table = TranspositionTable::new(17_000_000);
        perft_inner_with_table(root_node, depth, &mut table)
    } else {
        perft_inner(root_node, depth)
    }
}

fn perft_inner(node: &SearchNode, depth: i8) -> i64 {
    let mut nodes = 0;
    if depth == 1 {
        return node.pos.find_moves().len() as i64;
    }
    let move_list = node.pos.find_moves();
    for mv in move_list.iter() {
        let new_node = apply_move(&node, mv);
        nodes += perft_inner(&new_node, depth-1);
    }
    return nodes
}

fn perft_inner_with_table(
    node: &SearchNode, depth: i8, table: &mut TranspositionTable
) -> i64 {
    let mut nodes = 0;
    match table.get(node.key.0, depth) {
        Some(entry) => return entry.count,
        None => ()
    };
    if depth == 1 {
        return node.pos.find_moves().len() as i64;
    }
    let move_list = node.pos.find_moves();
    for mv in move_list.iter() {
        let new_node = apply_move(&node, mv);
        nodes += perft_inner_with_table(&new_node, depth-1, table);
    }
    table.set(node.key.0, nodes, depth);
    return nodes
}

/// Light perft test suite. Compares the number of nodes generated in these 
/// standard perft positions against the consensus. 
#[test_case(DEFAULT_FEN, vec![20, 400, 8902, 197281, 4865609], 5; "starting_position")]
#[test_case(POSITION_2, vec![48, 2039, 97862, 4085603], 4; "position_two")]
#[test_case(POSITION_3, vec![14, 191, 2812, 43238, 674624, 11030083], 6; "position_three")]
#[test_case(POSITION_4, vec![6, 264, 9467, 422333], 4; "position_four")]
#[test_case(POSITION_5, vec![44, 1486, 62379, 2103487], 4; "position_five")]
#[test_case(POSITION_6, vec![46, 2079, 89890, 3894594], 4; "position_six")]
fn light_perft_test(fen: &str, expected_nodes: Vec<i64>, depth: i8) {
    let node = SearchNode::new_from_fen(fen.to_string());
    for dpt in 1..depth + 1 {
        let result = perft(&node, dpt);
        assert_eq!(expected_nodes[dpt as usize - 1], result, "depth {}", dpt)
    }
}

/// Test suite for testing a variety of niche rules and mechanics found on
/// talk chess
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
fn talk_chess_perft_tests(fen: &str, depth: i8, expected_nodes: i64) {
    let node = SearchNode::new_from_fen(fen.to_string());
    assert_eq!(perft(&node, depth), expected_nodes);
}

// Not part of perft test suite, useful for debugging.
#[ignore]
#[test]
fn perft_debug() {
    let fen = "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1";
    let node = SearchNode::new_from_fen(fen.to_string());
    perft_divided(&node, 4);
}

/// Medium depth perft tests. Extension of the light perft test suite.
#[test_case(DEFAULT_FEN, vec![20, 400, 8902, 197281, 4865609, 119060324], 6; "starting_position")]
#[test_case(POSITION_2, vec![48, 2039, 97862, 4085603, 193690690], 5; "position_two")]
#[test_case(POSITION_3, vec![14, 191, 2812, 43238, 674624, 11030083, 178633661], 7; "position_three")]
#[test_case(POSITION_4, vec![6, 264, 9467, 422333, 15833292], 5; "position_four")]
#[test_case(POSITION_5, vec![44, 1486, 62379, 2103487, 89941194], 5; "position_five")]
#[test_case(POSITION_6, vec![46, 2079, 89890, 3894594, 164075551], 5; "position_six")]
fn medium_perft_test(fen: &str, expected_nodes: Vec<i64>, depth: i8) {
    let node = SearchNode::new_from_fen(fen.to_string());
    for dpt in 1..depth + 1 {
        let result = perft(&node, dpt);
        assert_eq!(expected_nodes[dpt as usize - 1], result, "depth {}", dpt)
    }
}

/// Highly intensive perft tests. Keep ignore flag to prevent from being 
/// run in a normal test suite.
#[ignore]
#[test_case(DEFAULT_FEN, 3195901860, 7; "starting_position")]
#[test_case(POSITION_2, 8031647685, 6; "position_two")]
#[test_case(POSITION_3, 3009794393, 8; "position_three")]
#[test_case(POSITION_4, 706045033, 6; "position_four")]
#[test_case(POSITION_6, 6923051137, 6; "position_six")]
fn deep_perft_test(fen: &str, expected_nodes: i64, depth: i8) {
    let node = SearchNode::new_from_fen(fen.to_string());
    let result = perft(&node, depth);
    assert_eq!(result, expected_nodes)
}