use super::*;
use crate::common::*;
use crate::common::bittools as bt;

/// The State trait is implemented by structs which contain methods which
/// are specific to the color. For example, black and white implement different
/// methods to push the pawn forward. This implementation circumvents the need
/// to repeatedly use if white / black logic

pub(crate) trait State {
    fn promotion_rank(&self) -> u64;
    fn ep_capture_rank(&self) -> u64;
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet;
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet;
    fn color(&self) -> Color;
    fn pawn_sgl_push_targets(&self, pos: &Position) -> u64;
    fn pawn_dbl_push_targets(&self, pos: &Position) -> u64;
    fn pawn_lcap_targets(&self, pos: &Position) -> u64;
    fn pawn_rcap_targets(&self, pos: &Position) -> u64;
    fn pawn_sgl_push_srcs(&self, targets: u64) -> u64;
    fn pawn_dbl_push_srcs(&self, targets: u64) -> u64;
    fn pawn_lcap_srcs(&self, targets: u64) -> u64;
    fn pawn_rcap_srcs(&self, targets: u64) -> u64;
    fn pawn_en_passant_srcs(&self, pos: &Position) -> u64;
    fn pawn_en_passant_cap(&self, pos: &Position) -> u64;
    fn kscm(&self) -> u64;
    fn qscms(&self) -> u64;
    fn qscmf(&self) -> u64;
    fn our_ksc(&self, pos: &Position) -> bool;
    fn our_sqc(&self, pos: &Position) -> bool;
    fn their_ksc(&self, pos: &Position) -> bool;
    fn their_qsc(&self, pos: &Position) -> bool;
    fn unsafe_squares_pawn(&self, pos: &Position) -> u64;
    fn pawn_checking_squares(&self, pos: &Position) -> u64;
    fn our_ks_rook_starting_sq(&self) -> u64;
    fn our_qs_rook_starting_sq(&self) -> u64;
    fn their_ks_rook_starting_sq(&self) -> u64;
    fn their_qs_rook_starting_sq(&self) -> u64;
    // Setters
    fn set_our_ksc(&self, data: &mut Data, value: bool);
    fn set_our_qsc(&self, data: &mut Data, value: bool);
    fn set_their_ksc(&self, data: &mut Data, value: bool);
    fn set_their_qsc(&self, data: &mut Data, value: bool);
    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;
    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;

}

impl State for White {

    fn promotion_rank(&self) -> u64 {RANK_8}
    fn ep_capture_rank(&self) -> u64 {RANK_5}
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.w_pieces}
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.b_pieces}
    fn our_ksc(&self, pos: &Position) -> bool {pos.data.w_kingside_castle}
    fn our_sqc(&self, pos: &Position) -> bool {pos.data.w_queenside_castle}
    fn their_ksc(&self, pos: &Position) -> bool {pos.data.b_kingside_castle}
    fn their_qsc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn kscm(&self) -> u64 {0x60}
    fn qscms(&self) -> u64 {0xc}
    fn qscmf(&self) -> u64 {0xe}
    fn color(&self) -> Color {Color::White}
    fn our_ks_rook_starting_sq(&self) -> u64 {WKROOK}
    fn our_qs_rook_starting_sq(&self) -> u64 {WQROOK}
    fn their_ks_rook_starting_sq(&self) -> u64 {BKROOK}
    fn their_qs_rook_starting_sq(&self) -> u64 {BQROOK}

    fn pawn_sgl_push_targets(&self, pos: &Position) -> u64 {
        bt::north_one(
            pos.data.w_pieces.pawn
        ) & pos.data.free
    }

    fn pawn_dbl_push_targets(&self, pos: &Position) -> u64 {
        let sgl_push = bt::north_one(
            pos.data.w_pieces.pawn & RANK_2
        ) & pos.data.free;
        bt::north_one(sgl_push) & pos.data.free
    }

    fn pawn_lcap_targets(&self, pos: &Position) -> u64 {
        bt::nort_west(
            pos.data.w_pieces.pawn
        ) & pos.data.b_pieces.any
    }

    fn pawn_rcap_targets(&self, pos: &Position) -> u64 {
        bt::nort_east(
            pos.data.w_pieces.pawn
        ) & pos.data.b_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: u64) -> u64 {
        bt::south_one(targets)
    }

    fn pawn_dbl_push_srcs(&self, targets: u64) -> u64 {
        bt::south_two(targets)
    }

    fn pawn_lcap_srcs(&self, targets: u64) -> u64 {
        bt::sout_east(targets)
    }

    fn pawn_rcap_srcs(&self, targets: u64) -> u64 {
        bt::sout_west(targets)
    }

    fn pawn_en_passant_srcs(&self, pos: &Position) -> u64 {
        (bt::sout_west(pos.data.en_passant_target_sq) 
            | bt::sout_east(pos.data.en_passant_target_sq))
            & pos.data.w_pieces.pawn
            & RANK_5
    }

    fn pawn_en_passant_cap(&self, pos: &Position) -> u64 {
        bt::south_one(pos.data.en_passant_target_sq)
    }

    fn unsafe_squares_pawn(&self, pos: &Position) -> u64 {
        bt::sout_west(pos.data.b_pieces.pawn) 
        | bt::sout_east(pos.data.b_pieces.pawn)
    }

    fn pawn_checking_squares(&self, pos: &Position) -> u64 {
        let king = pos.data.w_pieces.king;
        bt::nort_east(king) | bt::nort_west(king)
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
        data.b_kingside_castle = value
    }

    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet {
        &mut data.w_pieces
    }

    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet{
        &mut data.b_pieces
    }

}

impl State for Black {

    fn promotion_rank(&self) -> u64 {RANK_1}
    fn ep_capture_rank(&self) -> u64 {RANK_4}
    fn our_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.b_pieces}
    fn their_pieces<'a>(&'a self, pos: &'a Position) -> &PieceSet {&pos.data.w_pieces}
    fn our_ksc(&self, pos: &Position) -> bool {pos.data.b_kingside_castle}
    fn our_sqc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn their_ksc(&self, pos: &Position) -> bool {pos.data.w_kingside_castle}
    fn their_qsc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn kscm(&self) -> u64 {0x6000000000000000}
    fn qscms(&self) -> u64 {0xc00000000000000}
    fn qscmf(&self) -> u64 {0xe00000000000000}
    fn color(&self) -> Color {Color::Black}
    fn our_ks_rook_starting_sq(&self) -> u64 {BKROOK}
    fn our_qs_rook_starting_sq(&self) -> u64 {BQROOK}
    fn their_ks_rook_starting_sq(&self) -> u64 {WKROOK}
    fn their_qs_rook_starting_sq(&self) -> u64 {WQROOK}

    fn pawn_sgl_push_targets(&self, pos: &Position) -> u64 {
        bt::south_one(
            pos.data.b_pieces.pawn
        ) & pos.data.free
    }

    fn pawn_dbl_push_targets(&self, pos: &Position) -> u64 {
        let sgl_push: u64 = bt::south_one(
            pos.data.b_pieces.pawn & RANK_7
        ) & pos.data.free;
        bt::south_one(sgl_push) & pos.data.free
    }

    fn pawn_lcap_targets(&self, pos: &Position) -> u64 {
        bt::sout_west(
            pos.data.b_pieces.pawn
        ) & pos.data.w_pieces.any
    }

    fn pawn_rcap_targets(&self, pos: &Position) -> u64 {
        bt::sout_east(
            pos.data.b_pieces.pawn
        ) & pos.data.w_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: u64) -> u64 {
        bt::north_one(targets)
    }

    fn pawn_dbl_push_srcs(&self, targets: u64) -> u64 {
        bt::north_two(targets)
    }

    fn pawn_lcap_srcs(&self, targets: u64) -> u64 {
        bt::nort_east(targets)
    }

    fn pawn_rcap_srcs(&self, targets: u64) -> u64 {
        bt::nort_west(targets)
    }

    fn pawn_en_passant_srcs(&self, pos: &Position) -> u64 {
        (bt::nort_west(pos.data.en_passant_target_sq)
        | bt::nort_east(pos.data.en_passant_target_sq))
        & pos.data.b_pieces.pawn
        & RANK_4
    }

    fn pawn_en_passant_cap(&self, pos: &Position) -> u64 {
        bt::north_one(pos.data.en_passant_target_sq)
    }

    fn unsafe_squares_pawn(&self, pos: &Position) -> u64 {
        bt::nort_west(pos.data.w_pieces.pawn)
        | bt::nort_east(pos.data.w_pieces.pawn)
    }

    fn pawn_checking_squares(&self, pos: &Position) -> u64 {
        let king = pos.data.b_pieces.king;
        bt::sout_east(king) | bt::sout_west(king)
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