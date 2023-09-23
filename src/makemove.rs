/// Make move function for applying a move to a position
use super::*;

use mv::Move;
use position::states::*;
use types::{Color, MoveT, Piece};

impl position::Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&self, mv: &Move) -> Self {
        match self.stm {
            Color::White => self.make_move_inner::<White, Black>(mv),
            Color::Black => self.make_move_inner::<Black, White>(mv),
        }
    }

    #[inline(always)]
    fn make_move_inner<T: State, U: State>(&self, mv: &Move) -> Self {
        let mut new_pos = *self;
        // Unpack move data
        let to = mv.to();
        let from = mv.from();
        let mt = mv.mt();
        let captured_pt = new_pos.them.pt_at(to);
        let moved_pt = new_pos.us.pt_at(from).expect("is occupied");

        // Undo current ep key before position is modified
        new_pos.ep_key_update::<T>();

        // Increment clocks
        new_pos.halfmove_clock += 1;
        new_pos.fullmove_clock += new_pos.stm as u8;

        // Source squares must be free and target squares must be occupied
        new_pos.free |= from;
        new_pos.free &= !to;

        // Our bitboards must be flipped at the target and source
        let move_mask = from | to;
        new_pos.us[moved_pt] ^= move_mask;
        new_pos.us.all ^= move_mask;
        new_pos.move_key_update(moved_pt, from, to, new_pos.wtm);

        // Reset halfmove clock on pawn moves, remove castle rights on king moves
        match moved_pt {
            Piece::Pawn => new_pos.halfmove_clock = 0,
            Piece::King => new_pos.castling_rights &= !T::rank_1(),
            _ => (),
        }

        // If the rooks have moved, remove right to castle on that side
        new_pos.castling_rights &= !from;

        // Set ep target to empty, set later if dbl pawn push
        new_pos.ep_sq = constants::bb::EMPTY;

        // Captures, excluding en passant
        if let Some(pt) = captured_pt {
            new_pos.them[pt] ^= to;
            new_pos.them.all ^= to;
            new_pos.square_key_update(pt, to, !new_pos.wtm);
            // Remove castling right if rook has been captured
            new_pos.castling_rights &= !to;
            new_pos.halfmove_clock = 0;
        }

        // Promotions
        if mv.is_promo() {
            let promo_pt = mv.promo_pt();
            new_pos.us[promo_pt] ^= to;
            new_pos.us.pawn ^= to;
            new_pos.square_key_update(Piece::Pawn, to, new_pos.wtm);
            new_pos.square_key_update(promo_pt, to, new_pos.wtm);
        }

        // Execute special actions
        match mt {
            MoveT::DoublePawnPush => {
                // Ep target is one square behind dbl push target
                new_pos.ep_sq = T::back_one(to);
            }

            MoveT::KSCastle | MoveT::QSCastle => {
                let (rook_from, rook_to) = if let MoveT::KSCastle = mt {
                    (to.east_one(), to.west_one()) // Short Castle
                } else {
                    (to.west_two(), to.east_one()) // Long Castle
                };
                let mask = rook_from | rook_to;
                new_pos.us.rook ^= mask;
                new_pos.us.all ^= mask;
                new_pos.free ^= mask;
                new_pos.move_key_update(Piece::Rook, rook_from, rook_to, new_pos.wtm);
            }

            MoveT::EnPassant => {
                let ep_sq = T::back_one(to);
                new_pos.them.pawn ^= ep_sq;
                new_pos.them.all ^= ep_sq;
                new_pos.free ^= ep_sq;
                new_pos.square_key_update(Piece::Pawn, ep_sq, !new_pos.wtm);
            }

            _ => (),
        }

        new_pos.occ = !new_pos.free;
        // Change the turn and state
        new_pos.change_state();
        // Update key
        new_pos.turn_key_update();
        new_pos.ep_key_update::<U>();
        new_pos.castling_key_update(self.castling_rights);
        new_pos
    }
}
