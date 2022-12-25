use chess_engine::engine::{*, movelist::MoveList};
use test_case::test_case;
use common::*;
use position::Position;
use bitboard::BB;

#[test_case(
    POSITION_2, 21, 30,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P1Q1/2N4p/PPPBBPPP/R3K2R b KQkq - 1 1";
    "base case")]
#[test_case(
    "r3k2r/p2pqpb1/bn2pnp1/2pPN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R w KQkq c6 0 2", 14, 22,
    "r3k2r/p2pqpb1/bn2pnp1/2pPN3/Pp2P3/2N2QPp/1PPBBP1P/R3K2R b KQkq - 0 2";
    "loss of en passant")]
#[test_case(
    POSITION_2, 4, 5,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R4K1R b kq - 1 1";
    "loss of castling")]
fn test_hash_update_quiet(
    starting_pos: &str, src_sq: usize, target_sq: usize, expected_position: &str
) {
    let pos = Position::from_fen(starting_pos.to_string()).unwrap();
    // Specify move
    let mut move_list = MoveList::new();
    move_list.add_quiet_move(BB::from_index(target_sq), BB::from_index(src_sq));
    let mv = move_list.pop().unwrap();
    // Apply move
    let new_pos = pos.make_move(&mv);
    let expected_pos = Position::from_fen(expected_position.to_string()).unwrap();
    assert_eq!(new_pos.key.0, expected_pos.key.0)
}

#[test_case(POSITION_2, 8, 24,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1";
    "eps gain")]
#[test_case(POSITION_2, 14, 30,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P1P1/2N2Q1p/PPPBBP1P/R3K2R b KQkq g3 0 1";
    "no eps gain")]
#[test_case(
    "r3k2r/p1ppqpb1/bn2Pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1", 50, 34,
    "r3k2r/p2pqpb1/bn2Pnp1/2p1N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq c6 0 2";
    "no eps gain black")]
#[test_case(
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1", 50, 34,
    "r3k2r/p2pqpb1/bn2pnp1/2pPN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R w KQkq c6 0 2";
    "eps transfer")]
fn test_hash_update_double_pawn_push(
    starting_pos: &str, src_sq: usize, target_sq: usize, expected_position: &str
) {
    let pos = Position::from_fen(starting_pos.to_string()).unwrap();
    // Specify move
    let mut move_list = MoveList::new();
    move_list.add_double_pawn_push(BB::from_index(target_sq), BB::from_index(src_sq));
    let mv = move_list.pop().unwrap();
    // Apply move
    let new_pos = pos.make_move(&mv);
    let expected_pos = Position::from_fen(expected_position.to_string()).unwrap();
    assert_eq!(new_pos.key.0, expected_pos.key.0)
}

#[test_case(
    "r3k2r/p2pqpb1/bn2Pn2/2p1N1p1/1p2P3/1PN2Q1p/P1PBBPPP/R3K2R w KQkq - 0 3", 4, 6,
    "r3k2r/p2pqpb1/bn2Pn2/2p1N1p1/1p2P3/1PN2Q1p/P1PBBPPP/R4RK1 b kq - 1 3";
    "white")]
#[test_case(
    "r3k2r/p2pqpb1/bn2Pn2/2p1N1p1/1p2P3/1PN2Q1p/P1PBBPPP/R4RK1 b kq - 1 3", 60, 62,
    "r4rk1/p2pqpb1/bn2Pn2/2p1N1p1/1p2P3/1PN2Q1p/P1PBBPPP/R4RK1 w - - 2 4";
    "black")]
fn test_hash_update_castling(
    starting_pos: &str, src_sq: usize, target_sq: usize, expected_position: &str
) {
    let pos = Position::from_fen(starting_pos.to_string()).unwrap();
    // Specify move
    let mut move_list = MoveList::new();
    move_list.add_short_castle(BB::from_index(target_sq), BB::from_index(src_sq));
    let mv = move_list.pop().unwrap();
    // Apply move
    let new_pos = pos.make_move(&mv);
    let expected_pos = Position::from_fen(expected_position.to_string()).unwrap();
    assert_eq!(new_pos.key.0, expected_pos.key.0)
}

#[test_case(
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1", 25, 16,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/4P3/p1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2";
    "black")]
fn test_hash_update_en_passant(
    starting_pos: &str, src_sq: usize, target_sq: usize, expected_position: &str
) {
    let pos = Position::from_fen(starting_pos.to_string()).unwrap();
    // Specify move
    let mut move_list = MoveList::new();
    move_list.add_en_passant_capture(BB::from_index(target_sq), BB::from_index(src_sq));
    let mv = move_list.pop().unwrap();
    // Apply move
    let new_pos = pos.make_move(&mv);
    let expected_pos = Position::from_fen(expected_position.to_string()).unwrap();
    assert_eq!(new_pos.key.0, expected_pos.key.0)
}