/// Basic test suite finding piece-specific moves down to one ply
use super::*;

use constants::fen::*;
use test_case::test_case;

fn setup(fen: &str) -> (Position, movelist::UnorderedList) {
    (
        Position::from_fen(fen).unwrap(),
        movelist::UnorderedList::new(),
    )
}

#[test_case(START, 16; "starting")]
#[test_case(TEST_2, 8; "position_two")]
fn test_pawn_movegen(fen: &str, expected_nodes: usize) {
    let (position, mut movelist) = setup(fen);
    generate_all(&position, &mut movelist);
    let pawnmoves: Vec<_> = movelist
        .iter()
        .filter(|mv| matches!(position.us.piecetype_at(mv.from()), Some(PieceType::Pawn)))
        .collect();
    assert_eq!(expected_nodes, pawnmoves.len());
}

#[test_case(START, 4; "starting")]
#[test_case(TEST_2, 11; "position_two")]
fn test_knight_movegen(fen: &str, expected_nodes: usize) {
    let (position, mut movelist) = setup(fen);
    generate_all(&position, &mut movelist);
    let knightmoves: Vec<_> = movelist
        .iter()
        .filter(|mv| matches!(position.us.piecetype_at(mv.from()), Some(PieceType::Knight)))
        .collect();
    assert_eq!(expected_nodes, knightmoves.len());
}

#[test_case(START, 0; "starting")]
#[test_case(TEST_2, 2; "position_two")]
fn test_king_movegen(fen: &str, expected_nodes: usize) {
    let (position, mut movelist) = setup(fen);
    generate_king_moves(&position, &mut movelist, GeneratorType::NonEvasions);
    assert_eq!(expected_nodes, movelist.len());
}

#[test_case(START, 0; "starting")]
#[test_case(TEST_2, 2; "position_two")]
fn test_castling_movegen(fen: &str, expected_nodes: usize) {
    let (position, mut movelist) = setup(fen);
    generate_castles(&position, &mut movelist);
    assert_eq!(expected_nodes, movelist.len());
}

#[test_case(START, 20, 0; "starting")]
#[test_case(TEST_2, 48, 8; "position_two")]
#[test_case(TEST_3, 14, 1; "position_three")]
#[test_case(TEST_4, 6, 0; "position_four")]
fn test_movegen(fen: &str, expected_nodes: i32, expected_captures: usize) {
    let (position, mut movelist) = setup(fen);
    generate_all(&position, &mut movelist);
    let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
    assert_eq!(expected_nodes, movelist.len() as i32, "nodes");
    assert_eq!(expected_captures, captures.len(), "captures")
}

#[test_case(TEST_2, 8; "position_two")]
#[test_case(TEST_3, 1; "position_three")]
fn test_captures_movegen(fen: &str, expected_captures: usize) {
    let (position, mut movelist) = setup(fen);
    generate(GeneratorType::Captures, &position, &mut movelist);
    assert_eq!(movelist.len(), expected_captures);
    let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
    assert_eq!(captures.len(), expected_captures);
}

#[test]
fn test_evasion_movegen() {
    let (position, mut movelist) = setup("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1");
    generate(
        GeneratorType::Evasions(position.opponent_checkers()),
        &position,
        &mut movelist,
    );
    assert_eq!(movelist.len(), 11)
}
