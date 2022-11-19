use chess_engine::{*, common::*};

#[test]
fn test_position_new_from_fen_default() {
    let pos = position::Position::new_from_fen(common::DEFAULT_FEN.to_string());
    assert_eq!(pos.w_pieces[Piece::Any as usize], RANK_1 | RANK_2);
    assert_eq!(pos.w_pieces[Piece::Pawn as usize], RANK_2);
    assert_eq!(pos.w_pieces[Piece::Rook as usize], (1 << 0) | 1 << 7);
    assert_eq!(pos.w_pieces[Piece::Knight as usize], (1 << 1) | 1 << 6);
    assert_eq!(pos.w_pieces[Piece::Bishop as usize], (1 << 2) | 1 << 5);
    assert_eq!(pos.w_pieces[Piece::Queen as usize], 1 << 3);
    assert_eq!(pos.w_pieces[Piece::King as usize], 1 << 4);
    assert_eq!(pos.b_pieces[Piece::Any as usize], RANK_7 | RANK_8);
    assert_eq!(pos.b_pieces[Piece::Pawn as usize], RANK_7);
    assert_eq!(pos.b_pieces[Piece::Rook as usize], (1 << 56) | 1 << 63);
    assert_eq!(pos.b_pieces[Piece::Knight as usize], (1 << 57) | 1 << 62);
    assert_eq!(pos.b_pieces[Piece::Bishop as usize], (1 << 58) | 1 << 61);
    assert_eq!(pos.b_pieces[Piece::Queen as usize], 1 << 59);
    assert_eq!(pos.b_pieces[Piece::King as usize], 1 << 60);
    assert_eq!(pos.occ, RANK_1 | RANK_2 | RANK_7 | RANK_8);
    assert_eq!(pos.free, RANK_3 | RANK_4 | RANK_5 | RANK_6);
    assert_eq!(pos.white_to_move, true);
    assert_eq!(pos.w_kingside_castle, true);
    assert_eq!(pos.b_kingside_castle, true);
    assert_eq!(pos.w_queenside_castle, true);
    assert_eq!(pos.b_queenside_castle, true);
    assert_eq!(pos.en_passant_target_sq, 0);
    assert_eq!(pos.halfmove_clock, 0);
    assert_eq!(pos.fullmove_clock, 1);
}

#[test]
fn test_position_new_from_fen_eps() {
    let pos = position::Position::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq d6 0 1".to_string());
    assert_eq!(pos.en_passant_target_sq, 1 << 43);
}
