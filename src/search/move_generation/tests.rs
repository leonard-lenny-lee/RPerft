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
        let mut move_vec = Vec::new();
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::SinglePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
        assert_eq!(8, move_vec.len());
    }

    #[test]
    fn test_dbl_push_pawn_move_gen() {
        let pos = create_position();
        let mut move_vec = Vec::new();
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::DoublePush,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
        assert_eq!(8, move_vec.len());
    }

    #[test]
    fn test_push_lcap_move_gen() {
        let pos = create_position();
        let mut move_vec = Vec::new();
        find_pawn_moves(
            &mut move_vec,
            &pos, 
            PawnMove::CaptureLeft,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_push_rcap_move_gen() {
        let pos = create_position();
        let mut move_vec = Vec::new();
        find_pawn_moves(
            &mut move_vec,
            &pos,
            PawnMove::CaptureRight,
            FILLED_BB,
            FILLED_BB,
            EMPTY_BB
        );
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_knight_move_gen() {
        let pos = create_position();
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
        assert_eq!(4, move_vec.len());
    }

    #[test]
    fn test_king_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let mut move_vec = Vec::new();
        find_king_moves(
            &mut move_vec,
            &pos,
            &maps,
            EMPTY_BB
        );
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_bishop_move_gen() {
        let pos = create_position();
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
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_rook_move_gen() {
        let pos = create_position();
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
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_queen_move_gen() {
        let pos = create_position();
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
        assert_eq!(0, move_vec.len());
    }

    #[test]
    fn test_move_gen() {
        let pos = create_position();
        let maps = Maps::new();
        let move_vec = find_moves(&pos, &maps);
        assert_eq!(20, move_vec.len())
    }

}