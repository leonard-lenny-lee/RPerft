use super::*;
use crate::common::*;
use crate::common::bittools as bt;

pub(crate) trait State {

    fn promotion_rank(&self) -> u64;
    fn ep_capture_rank(&self) -> u64;
    fn our_pieces(&self, pos: &Position) -> &PieceSet;
    fn their_pieces(&self, pos: &Position) -> &PieceSet;
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
    fn qscm(&self) -> u64;
    fn ksc(&self, pos: &Position) -> bool;
    fn qsc(&self, pos: &Position) -> bool;

}

impl State for White {

    fn promotion_rank(&self) -> u64 {RANK_8}
    fn ep_capture_rank(&self) -> u64 {RANK_4}
    fn our_pieces(&self, pos: &Position) -> &PieceSet {&pos.data.w_pieces}
    fn their_pieces(&self, pos: &Position) -> &PieceSet {&pos.data.b_pieces}
    fn ksc(&self, pos: &Position) -> bool {pos.data.w_kingside_castle}
    fn qsc(&self, pos: &Position) -> bool {pos.data.w_queenside_castle}
    fn kscm(&self) -> u64 {0x60}
    fn qscm(&self) -> u64 {0xe}
    fn color(&self) -> Color {Color::White}

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

}

impl State for Black {

    fn promotion_rank(&self) -> u64 {RANK_1}
    fn ep_capture_rank(&self) -> u64 {RANK_5}
    fn our_pieces(&self, pos: &Position) -> &PieceSet {&pos.data.b_pieces}
    fn their_pieces(&self, pos: &Position) -> &PieceSet {&pos.data.w_pieces}
    fn ksc(&self, pos: &Position) -> bool {pos.data.b_kingside_castle}
    fn qsc(&self, pos: &Position) -> bool {pos.data.b_queenside_castle}
    fn kscm(&self) -> u64 {0x6000000000000000}
    fn qscm(&self) -> u64 {0xe00000000000000}
    fn color(&self) -> Color {Color::Black}

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
        ) & pos.data.w_pieces.pawn
    }

    fn pawn_rcap_targets(&self, pos: &Position) -> u64 {
        bt::sout_east(
            pos.data.b_pieces.pawn
        ) & pos.data.w_pieces.pawn
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

}