/// Make move function for applying a move to a position
use super::*;
use movelist::Move;
use position::Position;
use types::{
    MoveType::{self, *},
    PieceType::{self, *},
};

impl Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&mut self, mv: &Move) {
        // Unpack move data
        let to = mv.to();
        let from = mv.from();
        let mt = mv.movetype();
        let captured_pt = self.them.pt_at(to);
        let moved_pt = self.us.pt_at(from).expect("from must be occ");

        // Push to undo info stack
        self.unmake_info.push(UnmakeInfo {
            from,
            to,
            mt,
            moved_pt,
            captured_pt,
            castling_rights: self.castling_rights,
            halfmove_clock: self.halfmove_clock,
            ep_sq: self.ep_sq,
            key: self.key,
        });

        // Undo current ep key before position is modified
        self.ep_key_update();

        // Increment clocks
        self.ply += 1;
        self.halfmove_clock += 1;
        self.fullmove_clock += self.stm as u8;

        // Source squares must be free and target squares must be occupied
        self.free |= from;
        self.free &= !to;

        // Our bitboards must be flipped at the target and source
        let move_mask = from | to;
        self.us[moved_pt] ^= move_mask;
        self.us.all ^= move_mask;
        self.move_key_update(moved_pt, from, to, self.wtm());

        // Reset halfmove clock on pawn moves, remove castle rights on king moves
        match moved_pt {
            Pawn => self.halfmove_clock = 0,
            King => self.castling_rights &= !self.rank_1(),
            _ => (),
        }

        // If the rooks have moved, remove right to castle on that side
        self.castling_rights &= !from;

        // Set ep target to empty, set later if dbl pawn push
        self.ep_sq = EMPTY_BB;

        // Captures, excluding en passant
        if let Some(pt) = captured_pt {
            self.them[pt] ^= to;
            self.them.all ^= to;
            self.sq_key_update(pt, to, !self.wtm());
            // Remove castling right if rook has been captured
            self.castling_rights &= !to;
            self.halfmove_clock = 0;
        }

        // Promotions
        if mv.is_promotion() {
            let promo_pt = mv.promo_pt().expect("must encode pt");
            self.us[promo_pt] ^= to;
            self.us.pawn ^= to;
            self.sq_key_update(Pawn, to, self.wtm());
            self.sq_key_update(promo_pt, to, self.wtm());
        }

        // Execute special actions
        match mt {
            DoublePawnPush => {
                // Ep target is one square behind dbl push target
                self.ep_sq = self.push_back(to);
            }

            ShortCastle | LongCastle => {
                let (rook_from, rook_to) = if let ShortCastle = mt {
                    (to.east_one(), to.west_one())
                } else {
                    (to.west_two(), to.east_one())
                };
                let mask = rook_from | rook_to;
                self.us.rook ^= mask;
                self.us.all ^= mask;
                self.free ^= mask;
                self.move_key_update(Rook, rook_from, rook_to, self.wtm())
            }

            EnPassant => {
                let ep_sq = self.push_back(to);
                self.them.pawn ^= ep_sq;
                self.them.all ^= ep_sq;
                self.free ^= ep_sq;
                self.sq_key_update(Pawn, ep_sq, !self.wtm())
            }

            _ => (),
        }

        self.occ = !self.free;
        // Change the turn and state
        self.change_state();
        // Update key
        self.turn_key_update();
        self.ep_key_update();
        self.castle_key_update();
    }

    pub fn unmake_move(&mut self) {
        let prev = self.unmake_info.pop().unwrap();

        // Reverse clocks
        self.ply -= 1;
        self.halfmove_clock = prev.halfmove_clock;
        self.fullmove_clock -= 1 - self.stm as u8;

        // Restore info from the stack
        self.castling_rights = prev.castling_rights;
        self.ep_sq = prev.ep_sq;
        self.halfmove_clock = prev.halfmove_clock;
        self.key = prev.key;

        // Source square must now be occupied, free target square for now
        self.free |= prev.to;
        self.free &= !prev.from;

        // Reverse changes to our bbs
        let moved_pt = self.them.pt_at(prev.to).expect("must be occupied");
        let move_mask = prev.from | prev.to;
        self.them.all ^= move_mask;

        if moved_pt == prev.moved_pt {
            self.them[moved_pt] ^= move_mask;
        } else {
            // Must be a promotion
            self.them[moved_pt] ^= prev.to;
            self.them.pawn ^= prev.from;
        }

        // Undo capture
        if let Some(pt) = prev.captured_pt {
            self.us[pt] ^= prev.to;
            self.us.all ^= prev.to;
            self.free ^= prev.to;
        }

        // Undo special actions
        match prev.mt {
            ShortCastle | LongCastle => {
                let mask = if let ShortCastle = prev.mt {
                    prev.to.east_one() | prev.to.west_one()
                } else {
                    prev.to.west_two() | prev.to.east_one()
                };
                self.them.rook ^= mask;
                self.them.all ^= mask;
                self.free ^= mask;
            }

            EnPassant => {
                let ep_sq = self.push(prev.to);
                self.us.pawn ^= ep_sq;
                self.us.all ^= ep_sq;
                self.free ^= ep_sq;
            }

            _ => (),
        }

        self.occ = !self.free;
        self.change_state();
    }
}

pub struct UnmakeInfo {
    // Move info
    pub from: BB,
    pub to: BB,
    pub mt: MoveType,
    // Irretrievable info
    pub moved_pt: PieceType,
    pub captured_pt: Option<PieceType>,
    pub castling_rights: BB,
    pub ep_sq: BB,
    pub halfmove_clock: u8,
    pub key: u64,
}
