use super::*;
use crate::common::*;
use crate::common::bittools as bt;

pub(crate) trait Pos {

    // Getters
    fn promotion_rank(&self) -> u64;

    fn ep_capture_rank(&self) -> u64;

    fn kscm(&self) -> u64;

    fn qscm(&self) -> u64;

    fn our_pieces(&self) -> &PieceSet;

    fn their_pieces(&self) -> &PieceSet;

    fn ksc(&self) -> bool;

    fn qsc(&self) -> bool;

    fn position(&self) -> &Position;

    // Pawn masks
    /// Possible squares pawns can push to
    fn pawn_sgl_push_targets(&self) -> u64;
    /// Possible squares pawns can double push to
    fn pawn_dbl_push_targets(&self) -> u64;
    /// Possible squares pawns can capture left
    fn pawn_lf_cap_targets(&self) -> u64;
    /// Possible squares pawns can capture right
    fn pawn_rt_cap_targets(&self) -> u64;
    /// Pawns able to single push
    fn pawn_sgl_push_srcs(&self, targets: u64) -> u64;
    /// Pawns able to double push
    fn pawn_dbl_push_srcs(&self, targets: u64) -> u64;
    /// Pawns able to capture left
    fn pawn_lf_cap_srcs(&self, targets: u64) -> u64;
    /// Pawns able to capture right
    fn pawn_rt_cap_srcs(&self, targets: u64) -> u64;
    /// Pawns able to capture en passant
    fn pawn_en_passant_srcs(&self) -> u64;
    /// Opponent pawn captured by en passant
    fn pawn_en_passant_cap(&self) -> u64;

}

impl Pos for White {

    fn promotion_rank(&self) -> u64 {RANK_8}

    fn ep_capture_rank(&self) -> u64 {RANK_4}

    fn our_pieces(&self) -> &PieceSet {&self.pos.w_pieces}

    fn their_pieces(&self) -> &PieceSet {&self.pos.b_pieces}

    fn ksc(&self) -> bool {self.pos.w_kingside_castle}

    fn qsc(&self) -> bool {self.pos.w_queenside_castle}

    fn position(&self) -> &Position {&self.pos}

    fn kscm(&self) -> u64 {0x60}

    fn qscm(&self) -> u64 {0xe}

    fn pawn_sgl_push_targets(&self) -> u64 {
        bt::north_one(
            self.pos.w_pieces.pawn
        ) & self.pos.free
    }

    fn pawn_dbl_push_targets(&self) -> u64 {
        let sgl_push = bt::north_one(
            self.pos.w_pieces.pawn & RANK_2
        ) & self.pos.free;
        bt::north_one(sgl_push) & self.pos.free
    }

    fn pawn_lf_cap_targets(&self) -> u64 {
        bt::nort_west(
            self.pos.w_pieces.pawn
        ) & self.pos.b_pieces.any
    }

    fn pawn_rt_cap_targets(&self) -> u64 {
        bt::nort_east(
            self.pos.w_pieces.pawn
        ) & self.pos.b_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: u64) -> u64 {
        bt::south_one(targets)
    }

    fn pawn_dbl_push_srcs(&self, targets: u64) -> u64 {
        bt::south_two(targets)
    }

    fn pawn_lf_cap_srcs(&self, targets: u64) -> u64 {
        bt::sout_east(targets)
    }

    fn pawn_rt_cap_srcs(&self, targets: u64) -> u64 {
        bt::sout_west(targets)
    }

    fn pawn_en_passant_srcs(&self) -> u64 {
        (bt::sout_west(self.pos.en_passant_target_sq) 
            | bt::sout_east(self.pos.en_passant_target_sq))
            & self.pos.w_pieces.pawn
            & RANK_5
    }

    fn pawn_en_passant_cap(&self) -> u64 {
        bt::south_one(self.pos.en_passant_target_sq)
    }

}

impl Pos for Black {

    fn promotion_rank(&self) -> u64 {RANK_1}

    fn ep_capture_rank(&self) -> u64 {RANK_5}

    fn our_pieces(&self) -> &PieceSet {&self.pos.b_pieces}

    fn their_pieces(&self) -> &PieceSet {&self.pos.w_pieces}

    fn ksc(&self) -> bool {self.pos.b_kingside_castle}

    fn qsc(&self) -> bool {self.pos.b_queenside_castle}

    fn position(&self) -> &Position {&self.pos}

    fn kscm(&self) -> u64 {0x6000000000000000}

    fn qscm(&self) -> u64 {0xe00000000000000}

    fn pawn_sgl_push_targets(&self) -> u64 {
        bt::south_one(
            self.pos.b_pieces.pawn
        ) & self.pos.free
    }

    fn pawn_dbl_push_targets(&self) -> u64 {
        let sgl_push: u64 = bt::south_one(
            self.pos.b_pieces.pawn & RANK_7
        ) & self.pos.free;
        bt::south_one(sgl_push) & self.pos.free
    }

    fn pawn_lf_cap_targets(&self) -> u64 {
        bt::sout_west(
            self.pos.b_pieces.pawn
        ) & self.pos.w_pieces.pawn
    }

    fn pawn_rt_cap_targets(&self) -> u64 {
        bt::sout_east(
            self.pos.b_pieces.pawn
        ) & self.pos.w_pieces.pawn
    }

    fn pawn_en_passant_srcs(&self) -> u64 {
        (bt::nort_west(self.pos.en_passant_target_sq)
        | bt::nort_east(self.pos.en_passant_target_sq))
        & self.pos.b_pieces.pawn
        & RANK_4
    }

}