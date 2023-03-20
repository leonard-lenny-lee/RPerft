/// Make move function for applying a move to a position
use super::*;
use movelist::Move;
use position::Position;
use types::{MoveType::*, PieceType};

impl Position {
    /// Create a new position by applying move data to a position
    pub fn do_move(&self, mv: &Move) -> Position {
        // Create a copy of the current position to modify
        let mut new_pos = *self;

        // Unpack move data
        let target = mv.target();
        let src = mv.src();
        let movetype = mv.movetype();

        // Source squares must be free and target squares must be occupied
        new_pos.free |= src;
        new_pos.free &= !target;

        // Our bitboards must be flipped at the target and source
        let us = new_pos.mut_us();
        let moved_pt = us.pt_at(src);
        let move_mask = src | target;
        us[moved_pt] ^= move_mask;
        us.all ^= move_mask;

        // If our king has moved, remove all further rights to castle
        if matches!(moved_pt, PieceType::King) {
            new_pos.castling_rights &= !new_pos.rank_1();
        }

        // If the rooks have moved, remove rights to castle
        new_pos.castling_rights &= !src;

        // Increment half move clock; reset if capture or pawn move
        if mv.is_capture() || matches!(moved_pt, PieceType::Pawn) {
            new_pos.halfmove_clock = 0
        } else {
            new_pos.halfmove_clock += 1
        }

        // Increment full move clock if black has moved
        new_pos.fullmove_clock += !new_pos.wtm() as u8;

        // Set ep target to empty, set if dbl pawn push
        new_pos.ep_target_sq = EMPTY_BB;

        // Execute special actions
        match movetype {
            Quiet => (),
            DoublePawnPush => {
                // Ep target is one square behind dbl push target
                new_pos.ep_target_sq = new_pos.push_back(target);
            }
            Castle { is_long } => new_pos.exec_castle(is_long, target),
            Capture => new_pos.exec_capture(target),
            EnPassant => new_pos.exec_ep(target),
            Promotion(pt) => new_pos.exec_promo(pt, target),
            PromotionCapture(pt) => {
                new_pos.exec_promo(pt, target);
                new_pos.exec_capture(target)
            }
        }

        new_pos.occupied = !new_pos.free;
        // Change the turn and state
        new_pos.change_state();
        new_pos.update_key(moved_pt, src, target, self);
        return new_pos;
    }

    #[inline(always)]
    fn exec_capture(&mut self, target: BB) {
        let them = self.mut_them();
        let captured_pt = them.pt_at(target);
        them[captured_pt] ^= target;
        them.all ^= target;
        self.castling_rights &= !target;
        self.sq_key_update(captured_pt, target, !self.wtm())
    }

    #[inline(always)]
    fn exec_promo(&mut self, promo_pt: PieceType, target: BB) {
        let us = self.mut_us();
        us[promo_pt] ^= target;
        us.pawn ^= target;
        self.sq_key_update(PieceType::Pawn, target, self.wtm());
        self.sq_key_update(promo_pt, target, self.wtm());
    }

    #[inline(always)]
    fn exec_castle(&mut self, is_long: bool, target: BB) {
        let (rook_src, rook_target) = if is_long {
            (target.west_two(), target.east_one())
        } else {
            (target.east_one(), target.west_one())
        };
        let us = self.mut_us();
        let mask = rook_src | rook_target;
        us.rook ^= mask;
        us.all ^= mask;
        self.free ^= mask;
        self.move_key_update(PieceType::Rook, rook_src, rook_target, self.wtm())
    }

    #[inline(always)]
    fn exec_ep(&mut self, target: BB) {
        let ep_capture_sq = self.push_back(target);
        let them = self.mut_them();
        them.pawn ^= ep_capture_sq;
        them.all ^= ep_capture_sq;
        self.free ^= ep_capture_sq;
        self.sq_key_update(PieceType::Pawn, ep_capture_sq, !self.wtm())
    }
}
