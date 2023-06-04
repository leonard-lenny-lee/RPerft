use super::*;

use movelist::UnorderedList;
use test_case::test_case;

fn setup(fen: &str) -> (Position, UnorderedList) {
    return (Position::from_fen(fen).unwrap(), UnorderedList::new());
}

#[test_case(STARTPOS, 16; "starting")]
#[test_case(TPOS2, 8; "position_two")]
fn test_pawn_movegen(fen: &str, expected_nodes: usize) {
    let (pos, mut movelist) = setup(fen);
    generate_all(&pos, &mut movelist);
    let pawnmoves: Vec<_> = movelist
        .iter()
        .filter(|mv| matches!(pos.us.pt_at(mv.from()), Some(Pawn)))
        .collect();
    assert_eq!(expected_nodes, pawnmoves.len());
}

#[test_case(STARTPOS, 4; "starting")]
#[test_case(TPOS2, 11; "position_two")]
fn test_knight_movegen(fen: &str, expected_nodes: usize) {
    let (pos, mut movelist) = setup(fen);
    generate_all(&pos, &mut movelist);
    let knightmoves: Vec<_> = movelist
        .iter()
        .filter(|mv| matches!(pos.us.pt_at(mv.from()), Some(Knight)))
        .collect();
    assert_eq!(expected_nodes, knightmoves.len());
}

#[test_case(STARTPOS, 0; "starting")]
#[test_case(TPOS2, 2; "position_two")]
fn test_king_movegen(fen: &str, expected_nodes: usize) {
    let (pos, mut movelist) = setup(fen);
    generate_king_moves(&pos, &mut movelist, GenType::NonEvasions);
    assert_eq!(expected_nodes, movelist.len());
}

#[test_case(STARTPOS, 0; "starting")]
#[test_case(TPOS2, 2; "position_two")]
fn test_castling_movegen(fen: &str, expected_nodes: usize) {
    let (pos, mut movelist) = setup(fen);
    generate_castles(&pos, &mut movelist);
    assert_eq!(expected_nodes, movelist.len());
}

#[test_case(STARTPOS, 20, 0; "starting")]
#[test_case(TPOS2, 48, 8; "position_two")]
#[test_case(TPOS3, 14, 1; "position_three")]
#[test_case(TPOS4, 6, 0; "position_four")]
fn test_movegen(fen: &str, expected_nodes: i32, expected_captures: usize) {
    let (pos, mut movelist) = setup(fen);
    generate_all(&pos, &mut movelist);
    let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
    assert_eq!(expected_nodes, movelist.len() as i32, "nodes");
    assert_eq!(expected_captures, captures.len(), "captures")
}

#[test_case(TPOS2, 8; "position_two")]
#[test_case(TPOS3, 1; "position_three")]
fn test_captures_movegen(fen: &str, expected_captures: usize) {
    let (pos, mut movelist) = setup(fen);
    generate(GenType::Captures, &pos, &mut movelist);
    assert_eq!(movelist.len(), expected_captures);
    let captures: Vec<_> = movelist.iter().filter(|x| x.is_capture()).collect();
    assert_eq!(captures.len(), expected_captures);
}

#[test]
fn test_evasion_movegen() {
    let (pos, mut movelist) = setup("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1");
    generate(GenType::Evasions(pos.checkers()), &pos, &mut movelist);
    assert_eq!(movelist.len(), 11)
}
