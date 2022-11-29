use super::*;
use bt::squares_to_bitboard;

use test_case::test_case;

fn generate_targets(move_vec: Vec<Move>) -> u64 {
    let mut targets = EMPTY_BB;
    for mv in move_vec {
        targets |= mv.target;
    }
    return targets
}

#[test_case(DEFAULT_FEN, 8, vec![16, 17, 18, 19, 20, 21, 22, 23]; "starting")]
#[test_case(POSITION_2, 4, vec![16, 17, 43, 22]; "position_two")]
fn test_sgl_push_pawn_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let mut move_vec = Vec::new();
    find_pawn_moves(
        &mut move_vec,
        &pos,
        PawnMove::SinglePush,
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets);
}

#[test_case(DEFAULT_FEN, 8, vec![24, 25, 26, 27, 28, 29, 30, 31]; "starting")]
#[test_case(POSITION_2, 2, vec![24, 30]; "position_two")]
fn test_dbl_push_pawn_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let mut move_vec = Vec::new();
    find_pawn_moves(
        &mut move_vec,
        &pos,
        PawnMove::DoublePush,
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}
#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 0, vec![]; "position_two")]
fn test_push_lcap_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let mut move_vec = Vec::new();
    find_pawn_moves(
        &mut move_vec,
        &pos, 
        PawnMove::CaptureLeft,
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 2, vec![44, 23]; "position_two")]
fn test_push_rcap_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let mut move_vec = Vec::new();
    find_pawn_moves(
        &mut move_vec,
        &pos,
        PawnMove::CaptureRight,
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}
#[test_case(DEFAULT_FEN, 4, vec![16, 18, 21, 23]; "starting")]
#[test_case(POSITION_2, 11, vec![1, 24, 33, 3, 51, 42, 26, 19, 30, 46, 53]; "position_two")]
fn test_knight_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_knight_moves(
        &mut move_vec,
        &pos,
        &maps,
        FILLED_BB,
        FILLED_BB, 
        EMPTY_BB,
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 2, vec![3, 5]; "position_two")]
fn test_king_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_king_moves(
        &mut move_vec,
        &pos,
        &maps,
        EMPTY_BB
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 11, vec![2, 20, 29, 38, 47, 3, 5, 19, 26, 33, 40]; "position_two")]
fn test_bishop_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_sliding_moves(
        &mut move_vec,
        &pos,
        SlidingPiece::Bishop,
        &maps, 
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB, 
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 5, vec![1, 2, 3, 5, 6]; "position_two")]
fn test_rook_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_sliding_moves(
        &mut move_vec,
        &pos,
        SlidingPiece::Rook,
        &maps, 
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB, 
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 9, vec![19, 20, 22, 23, 29, 37, 45, 30, 39]; "position_two")]
fn test_queen_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_sliding_moves(
        &mut move_vec,
        &pos,
        SlidingPiece::Queen,
        &maps, 
        FILLED_BB,
        FILLED_BB,
        EMPTY_BB, 
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 0, vec![]; "position_two")]
fn test_en_passant_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let mut move_vec = Vec::new();
    find_en_passant_moves(
        &mut move_vec,
        &pos,
        FILLED_BB,
        FILLED_BB,
        &maps,
        EMPTY_BB,
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
#[test_case(POSITION_2, 2, vec![2, 6]; "position_two")]
fn test_castling_move_gen(
    fen: &str, expected_nodes: i32, expected_targets: Vec<i32>
) {
    let pos = Position::new_from_fen(fen.to_string());
    let mut move_vec = Vec::new();
    find_castling_moves(
        &mut move_vec,
        &pos,
        EMPTY_BB,
    );
    assert_eq!(expected_nodes, move_vec.len() as i32);
    let targets = generate_targets(move_vec);
    let expected_targets = squares_to_bitboard(expected_targets);
    assert_eq!(expected_targets, targets)
}

#[test_case(DEFAULT_FEN, 20, 0; "starting")]
#[test_case(POSITION_2, 48, 8; "position_two")]
#[test_case(POSITION_3, 14, 1; "position_three")]
#[test_case(POSITION_4, 6, 0; "position_four")]
fn test_move_gen(
    fen: &str, expected_nodes: i32, expected_captures: i32,
) {
    let pos = Position::new_from_fen(fen.to_string());
    let maps = Maps::new();
    let move_vec = find_moves(&pos, &maps);
    let mut n_captures = 0;
    for mv in &move_vec {
        if mv.is_capture {
            n_captures += 1
        }
    }
    assert_eq!(expected_nodes, move_vec.len() as i32, "nodes");
    assert_eq!(expected_captures, n_captures, "captures")
}