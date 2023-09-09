/// Make move function for applying a move to a position
use super::*;
use move_::Move;
use types::{MoveType::*, Piece::*};

impl position::Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&self, mv: &Move) -> Self {
        let mut new_position = *self;
        // Unpack move data
        let to = mv.to();
        let from = mv.from();
        let mt = mv.movetype();
        let captured_pt = new_position.them.piecetype_at(to);
        let moved_pt = new_position.us.piecetype_at(from).expect("is occupied");

        // Undo current ep key before position is modified
        new_position.en_passant_key_update();

        // Increment clocks
        new_position.halfmove_clock += 1;
        new_position.fullmove_clock += new_position.side_to_move as u8;

        // Source squares must be free and target squares must be occupied
        new_position.free |= from;
        new_position.free &= !to;

        // Our bitboards must be flipped at the target and source
        let move_mask = from | to;
        new_position.us[moved_pt] ^= move_mask;
        new_position.us.all ^= move_mask;
        new_position.move_key_update(moved_pt, from, to, new_position.white_to_move);

        // Reset halfmove clock on pawn moves, remove castle rights on king moves
        match moved_pt {
            Pawn => new_position.halfmove_clock = 0,
            King => new_position.castling_rights &= !new_position.rank_1(),
            _ => (),
        }

        // If the rooks have moved, remove right to castle on that side
        new_position.castling_rights &= !from;

        // Set ep target to empty, set later if dbl pawn push
        new_position.en_passant = constants::bb::EMPTY;

        // Captures, excluding en passant
        if let Some(pt) = captured_pt {
            new_position.them[pt] ^= to;
            new_position.them.all ^= to;
            new_position.square_key_update(pt, to, !new_position.white_to_move);
            // Remove castling right if rook has been captured
            new_position.castling_rights &= !to;
            new_position.halfmove_clock = 0;
        }

        // Promotions
        if mv.is_promotion() {
            let promo_pt = mv.promotion_piecetype();
            new_position.us[promo_pt] ^= to;
            new_position.us.pawn ^= to;
            new_position.square_key_update(Pawn, to, new_position.white_to_move);
            new_position.square_key_update(promo_pt, to, new_position.white_to_move);
        }

        // Execute special actions
        match mt {
            DoublePawnPush => {
                // Ep target is one square behind dbl push target
                new_position.en_passant = new_position.back_one(to);
            }

            ShortCastle | LongCastle => {
                let (rook_from, rook_to) = if let ShortCastle = mt {
                    (to.east_one(), to.west_one())
                } else {
                    (to.west_two(), to.east_one())
                };
                let mask = rook_from | rook_to;
                new_position.us.rook ^= mask;
                new_position.us.all ^= mask;
                new_position.free ^= mask;
                new_position.move_key_update(Rook, rook_from, rook_to, new_position.white_to_move);
            }

            EnPassant => {
                let ep_sq = new_position.back_one(to);
                new_position.them.pawn ^= ep_sq;
                new_position.them.all ^= ep_sq;
                new_position.free ^= ep_sq;
                new_position.square_key_update(Pawn, ep_sq, !new_position.white_to_move);
            }

            _ => (),
        }

        new_position.occupied = !new_position.free;
        // Change the turn and state
        new_position.change_state();
        // Update key
        new_position.turn_key_update();
        new_position.en_passant_key_update();
        new_position.castling_key_update(self.castling_rights);
        new_position
    }
}
