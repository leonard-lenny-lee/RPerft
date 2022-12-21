use super::*;

/// The State trait is implemented by structs which contain methods which
/// are specific to the color. For example, black and white implement different
/// methods to push the pawn forward. This implementation circumvents the need
/// to repeatedly use if white / black logic

pub(crate) trait State {
    fn promotion_rank(&self) -> BB;
    fn ep_capture_rank(&self) -> BB;
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet;
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet;
    fn color(&self) -> Color;
    fn pawn_sgl_push(&self, src: BB) -> BB;
    fn pawn_dbl_push(&self, src: BB) -> BB;
    fn pawn_left_capture(&self, src: BB) -> BB;
    fn pawn_right_capture(&self, src: BB) -> BB;
    fn pawn_sgl_push_targets(&self, pos: &Position) -> BB;
    fn pawn_dbl_push_targets(&self, pos: &Position) -> BB;
    fn pawn_lcap_targets(&self, pos: &Position) -> BB;
    fn pawn_rcap_targets(&self, pos: &Position) -> BB;
    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB;
    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB;
    fn pawn_lcap_srcs(&self, targets: BB) -> BB;
    fn pawn_rcap_srcs(&self, targets: BB) -> BB;
    fn pawn_en_passant_srcs(&self, pos: &Position) -> BB;
    fn pawn_en_passant_cap(&self, pos: &Position) -> BB;
    fn kingside_castle_mask(&self) -> BB;
    fn queenside_castle_mask_safe(&self) -> BB;
    fn queenside_castle_mask_free(&self) -> BB;
    fn our_ksc(&self, pos: &Position) -> bool;
    fn our_sqc(&self, pos: &Position) -> bool;
    fn their_ksc(&self, pos: &Position) -> bool;
    fn their_qsc(&self, pos: &Position) -> bool;
    fn unsafe_squares_pawn(&self, pos: &Position) -> BB;
    fn pawn_checking_squares(&self, pos: &Position) -> BB;
    fn our_ks_rook_starting_sq(&self) -> BB;
    fn our_qs_rook_starting_sq(&self) -> BB;
    fn their_ks_rook_starting_sq(&self) -> BB;
    fn their_qs_rook_starting_sq(&self) -> BB;
    // Setters
    fn set_our_ksc(&self, data: &mut Data, value: bool);
    fn set_our_qsc(&self, data: &mut Data, value: bool);
    fn set_their_ksc(&self, data: &mut Data, value: bool);
    fn set_their_qsc(&self, data: &mut Data, value: bool);
    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;
    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;

}

impl State for White {

    fn promotion_rank(&self) -> BB {RANK_7}
    fn ep_capture_rank(&self) -> BB {RANK_5}
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.w_pieces}
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.b_pieces}
    fn our_ksc(&self, pos: &Position) -> bool {pos.data.w_kingside_castle}
    fn our_sqc(&self, pos: &Position) -> bool {pos.data.w_queenside_castle}
    fn their_ksc(&self, pos: &Position) -> bool {pos.data.b_kingside_castle}
    fn their_qsc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn kingside_castle_mask(&self) -> BB {BB(0x60)}
    fn queenside_castle_mask_safe(&self) -> BB {BB(0xc)}
    fn queenside_castle_mask_free(&self) -> BB {BB(0xe)}
    fn color(&self) -> Color {Color::White}
    fn our_ks_rook_starting_sq(&self) -> BB {W_KINGSIDE_ROOK_STARTING_SQ}
    fn our_qs_rook_starting_sq(&self) -> BB {W_QUEENSIDE_ROOK_STARTING_SQ}
    fn their_ks_rook_starting_sq(&self) -> BB {B_KINGSIDE_ROOK_STARTING_SQ}
    fn their_qs_rook_starting_sq(&self) -> BB {B_QUEENSIDE_ROOK_STARTING_SQ}

    fn pawn_sgl_push(&self, src: BB) -> BB {
        src.north_one()
    }

    fn pawn_dbl_push(&self, src: BB) -> BB {
        src.north_two()
    }

    fn pawn_left_capture(&self, src: BB) -> BB {
        src.nort_west()
    }

    fn pawn_right_capture(&self, src: BB) -> BB {
        src.nort_east()
    }

    fn pawn_sgl_push_targets(&self, pos: &Position) -> BB {
        pos.data.w_pieces.pawn.north_one() & pos.data.free
    }

    fn pawn_dbl_push_targets(&self, pos: &Position) -> BB {
        let sgl_push = (pos.data.w_pieces.pawn & RANK_2).north_one() & pos.data.free;
        sgl_push.north_one() & pos.data.free
    }

    fn pawn_lcap_targets(&self, pos: &Position) -> BB {
        pos.data.w_pieces.pawn.nort_west() & pos.data.b_pieces.any
    }

    fn pawn_rcap_targets(&self, pos: &Position) -> BB {
        pos.data.w_pieces.pawn.nort_east() & pos.data.b_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        targets.south_one()
    }

    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        targets.south_two()
    }

    fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        targets.sout_east()
    }

    fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        targets.sout_west()
    }

    fn pawn_en_passant_srcs(&self, pos: &Position) -> BB {
        (pos.data.en_passant_target_sq.sout_west()
            | pos.data.en_passant_target_sq.sout_east())
            & pos.data.w_pieces.pawn
            & RANK_5
    }

    fn pawn_en_passant_cap(&self, pos: &Position) -> BB {
        pos.data.en_passant_target_sq.south_one()
    }

    fn unsafe_squares_pawn(&self, pos: &Position) -> BB {
        pos.data.b_pieces.pawn.sout_west() | pos.data.b_pieces.pawn.sout_east()
    }

    fn pawn_checking_squares(&self, pos: &Position) -> BB {
        let king = pos.data.w_pieces.king;
        king.nort_east() | king.nort_west()
    }

    fn set_our_ksc(&self, data: &mut Data, value: bool) {
        data.w_kingside_castle = value
    }

    fn set_our_qsc(&self, data: &mut Data, value: bool) {
        data.w_queenside_castle = value
    }

    fn set_their_ksc(&self, data: &mut Data, value: bool) {
        data.b_kingside_castle = value
    }

    fn set_their_qsc(&self, data: &mut Data, value: bool) {
        data.b_queenside_castle = value
    }

    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet {
        &mut data.w_pieces
    }

    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet{
        &mut data.b_pieces
    }

}

impl State for Black {

    fn promotion_rank(&self) -> BB {RANK_2}
    fn ep_capture_rank(&self) -> BB {RANK_4}
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.b_pieces}
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.w_pieces}
    fn our_ksc(&self, pos: &Position) -> bool {pos.data.b_kingside_castle}
    fn our_sqc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn their_ksc(&self, pos: &Position) -> bool {pos.data.w_kingside_castle}
    fn their_qsc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn kingside_castle_mask(&self) -> BB {BB(0x6000000000000000)}
    fn queenside_castle_mask_safe(&self) -> BB {BB(0xc00000000000000)}
    fn queenside_castle_mask_free(&self) -> BB {BB(0xe00000000000000)}
    fn color(&self) -> Color {Color::Black}
    fn our_ks_rook_starting_sq(&self) -> BB {B_KINGSIDE_ROOK_STARTING_SQ}
    fn our_qs_rook_starting_sq(&self) -> BB {B_QUEENSIDE_ROOK_STARTING_SQ}
    fn their_ks_rook_starting_sq(&self) -> BB {W_KINGSIDE_ROOK_STARTING_SQ}
    fn their_qs_rook_starting_sq(&self) -> BB {W_QUEENSIDE_ROOK_STARTING_SQ}

    fn pawn_sgl_push(&self, src: BB) -> BB {
        src.south_one()
    }

    fn pawn_dbl_push(&self, src: BB) -> BB {
        src.south_two()
    }

    fn pawn_left_capture(&self, src: BB) -> BB {
        src.sout_west()
    }

    fn pawn_right_capture(&self, src: BB) -> BB {
        src.sout_east()
    }

    fn pawn_sgl_push_targets(&self, pos: &Position) -> BB {
        pos.data.b_pieces.pawn.south_one() & pos.data.free
    }

    fn pawn_dbl_push_targets(&self, pos: &Position) -> BB {
        let sgl_push = (pos.data.b_pieces.pawn & RANK_7).south_one() & pos.data.free;
        sgl_push.south_one() & pos.data.free
    }

    fn pawn_lcap_targets(&self, pos: &Position) -> BB {
        pos.data.b_pieces.pawn.sout_west() & pos.data.w_pieces.any
    }

    fn pawn_rcap_targets(&self, pos: &Position) -> BB {
        pos.data.b_pieces.pawn.sout_east() & pos.data.w_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        targets.north_one()
    }

    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        targets.north_two()
    }

    fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        targets.nort_east()
    }

    fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        targets.nort_west()
    }

    fn pawn_en_passant_srcs(&self, pos: &Position) -> BB {
        (pos.data.en_passant_target_sq.nort_west()
        | pos.data.en_passant_target_sq.nort_east())
        & pos.data.b_pieces.pawn
        & RANK_4
    }

    fn pawn_en_passant_cap(&self, pos: &Position) -> BB {
        pos.data.en_passant_target_sq.north_one()
    }

    fn unsafe_squares_pawn(&self, pos: &Position) -> BB {
        pos.data.w_pieces.pawn.nort_west() | pos.data.w_pieces.pawn.nort_east()
    }

    fn pawn_checking_squares(&self, pos: &Position) -> BB {
        let king = pos.data.b_pieces.king;
        king.sout_east() | king.sout_west()
    }

    fn set_our_ksc(&self, data: &mut Data, value: bool) {
        data.b_kingside_castle = value;
    }

    fn set_our_qsc(&self, data: &mut Data, value: bool) {
        data.b_queenside_castle = value
    }

    fn set_their_ksc(&self, data: &mut Data, value: bool) {
        data.w_kingside_castle = value
    }

    fn set_their_qsc(&self, data: &mut Data, value: bool) {
        data.w_queenside_castle = value
    }

    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet {
        &mut data.b_pieces
    }

    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet{
        &mut data.w_pieces
    }

}