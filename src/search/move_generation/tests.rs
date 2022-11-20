mod default_position_tests {

    use crate::global::maps::Maps;
    use crate::position::Position;
    use crate::common::*;
    use crate::search::move_generation::*;


    fn create_position() -> Position {
        return Position::new_from_fen(DEFAULT_FEN.to_string())
    }

    #[test]
    fn test_sgl_push_pawn_move_gen() {
        let pos = create_position();
        let n_moves = generate_pawn_moves(
            &pos,
            PawnMove::SinglePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        ).len();
        assert_eq!(8, n_moves);
    }

    #[test]
    fn test_dbl_push_pawn_move_gen() {
        let pos = create_position();
        let n_moves = generate_pawn_moves(
            &pos,
            PawnMove::DoublePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        ).len();
        assert_eq!(8, n_moves);
    }

    #[test]
    fn test_push_lcap_move_gen() {
        let pos = create_position();
        let n_moves = generate_pawn_moves(
            &pos, 
            PawnMove::CaptureLeft,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_push_rcap_move_gen() {
        let pos = create_position();
        let n_moves = generate_pawn_moves(
            &pos,
            PawnMove::CaptureRight,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_knight_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_jumping_moves(
            &pos,
            JumpingPiece::Knight,
            &pos.w_pieces,
            &maps, 
            EMPTY_BB,
            FILLED_BB,
            FILLED_BB, 
            EMPTY_BB
        ).len();
        assert_eq!(4, n_moves);
    }

    #[test]
    fn test_king_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_jumping_moves(
            &pos,
            JumpingPiece::King,
            &pos.w_pieces,
            &maps, 
            EMPTY_BB,
            FILLED_BB,
            FILLED_BB, 
            EMPTY_BB
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_bishop_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_sliding_moves(
            &pos,
            SlidingPiece::Bishop,
            &pos.w_pieces,
            &maps, 
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_rook_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_sliding_moves(
            &pos,
            SlidingPiece::Rook,
            &pos.w_pieces,
            &maps, 
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_queen_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_sliding_moves(
            &pos,
            SlidingPiece::Queen,
            &pos.w_pieces,
            &maps, 
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB, 
        ).len();
        assert_eq!(0, n_moves);
    }

    #[test]
    fn test_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let n_moves = generate_moves(&pos, &maps).len();
        assert_eq!(20, n_moves)
    }

}