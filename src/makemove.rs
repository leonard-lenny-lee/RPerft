/// Make move function for applying a move to a position
use super::*;
use movelist::Move;
use position::Position;
use types::{
    MoveType::{self, *},
    PieceType,
};

impl Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&self, mv: &Move) -> Position {
        // Create a copy of the current position to modify
        let mut new_pos = *self;
        new_pos.ply += 1;

        // Unpack move data
        let target = mv.to();
        let src = mv.from();
        let mt = mv.movetype();

        // Source squares must be free and target squares must be occupied
        new_pos.free |= src;
        new_pos.free &= !target;

        // Our bitboards must be flipped at the target and source
        let moved_pt = new_pos.us.pt_at(src).expect("from must be occ");
        let move_mask = src | target;
        new_pos.us[moved_pt] ^= move_mask;
        new_pos.us.all ^= move_mask;

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
        new_pos.ep_sq = EMPTY_BB;

        // Execute special actions
        match mt {
            Quiet => (),
            DoublePawnPush => {
                // Ep target is one square behind dbl push target
                new_pos.ep_sq = new_pos.push_back(target);
            }
            ShortCastle | LongCastle => new_pos.exec_castle(mt, target),
            Capture => new_pos.exec_capture(target),
            EnPassant => new_pos.exec_ep(target),
            NPromotion | BPromotion | RPromotion | QPromotion => {
                let promo_pt = mv.promo_pt().expect("must encode pt");
                new_pos.exec_promo(promo_pt, target)
            }
            NPromoCapture | BPromoCapture | RPromoCapture | QPromoCapture => {
                let promo_pt = mv.promo_pt().expect("must encode pt");
                new_pos.exec_promo(promo_pt, target);
                new_pos.exec_capture(target)
            }
        }

        new_pos.occ = !new_pos.free;
        // Change the turn and state
        new_pos.change_state();
        new_pos.update_key(moved_pt, src, target, self);
        return new_pos;
    }

    #[inline(always)]
    fn exec_capture(&mut self, target: BB) {
        let captured_pt = self.them.pt_at(target).unwrap();
        self.them[captured_pt] ^= target;
        self.them.all ^= target;
        self.castling_rights &= !target;
        self.sq_key_update(captured_pt, target, !self.wtm())
    }

    #[inline(always)]
    fn exec_promo(&mut self, promo_pt: PieceType, target: BB) {
        self.us[promo_pt] ^= target;
        self.us.pawn ^= target;
        self.sq_key_update(PieceType::Pawn, target, self.wtm());
        self.sq_key_update(promo_pt, target, self.wtm());
    }

    #[inline(always)]
    fn exec_castle(&mut self, mt: MoveType, target: BB) {
        let (rook_src, rook_target) = match mt {
            MoveType::ShortCastle => (target.east_one(), target.west_one()),
            MoveType::LongCastle => (target.west_two(), target.east_one()),
            _ => return,
        };
        let mask = rook_src | rook_target;
        self.us.rook ^= mask;
        self.us.all ^= mask;
        self.free ^= mask;
        self.move_key_update(PieceType::Rook, rook_src, rook_target, self.wtm())
    }

    #[inline(always)]
    fn exec_ep(&mut self, target: BB) {
        let ep_capture_sq = self.push_back(target);
        self.them.pawn ^= ep_capture_sq;
        self.them.all ^= ep_capture_sq;
        self.free ^= ep_capture_sq;
        self.sq_key_update(PieceType::Pawn, ep_capture_sq, !self.wtm())
    }
}
