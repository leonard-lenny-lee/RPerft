/// Make move function for applying a move to a position
use super::*;
use movelist::Move;
use position::{Position, StackData};
use types::{
    MoveType::*,
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

        // Push data to stack
        self.stack.push(StackData {
            from,
            to,
            mt,
            moved_pt,
            captured_pt,
            castling_rights: self.castling_rights,
            halfmove_clock: self.halfmove_clock,
            ep_sq: self.ep_sq,
            key: self.key,
            nnue_pos: self.nnue_pos,
        });

        // NNUE routine
        let from_sq = from.to_sq();
        let to_sq = to.to_sq();
        self.nnue_pos.move_pc(from_sq, to_sq);

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
        self.move_key_update(moved_pt, from, to, self.wtm);

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
            self.sq_key_update(pt, to, !self.wtm);
            // Remove castling right if rook has been captured
            self.castling_rights &= !to;
            self.halfmove_clock = 0;
            // Index swap has already occurred so from_sq actually points to the target pc
            self.nnue_pos.remove_pc(from_sq);
        }

        // Promotions
        if mv.is_promotion() {
            let promo_pt = mv.promo_pt().expect("must encode pt");
            self.us[promo_pt] ^= to;
            self.us.pawn ^= to;
            self.sq_key_update(Pawn, to, self.wtm);
            self.sq_key_update(promo_pt, to, self.wtm);
            self.nnue_pos.promote_pc(to_sq, promo_pt);
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
                self.move_key_update(Rook, rook_from, rook_to, self.wtm);
                self.nnue_pos.move_pc(rook_from.to_sq(), rook_to.to_sq());
            }

            EnPassant => {
                let ep_sq = self.push_back(to);
                self.them.pawn ^= ep_sq;
                self.them.all ^= ep_sq;
                self.free ^= ep_sq;
                self.sq_key_update(Pawn, ep_sq, !self.wtm);
                self.nnue_pos.remove_pc(ep_sq.to_sq());
            }

            _ => (),
        }

        self.occ = !self.free;
        // Change the turn and state
        self.nnue_pos.player ^= 1;
        self.change_state();
        // Update key
        self.turn_key_update();
        self.ep_key_update();
        self.castle_key_update();
    }

    pub fn unmake_move(&mut self) {
        let prev = self.stack.pop().unwrap();

        // Reverse clocks
        self.ply -= 1;
        self.halfmove_clock = prev.halfmove_clock;
        self.fullmove_clock -= 1 - self.stm as u8;

        // Restore info from the stack
        self.castling_rights = prev.castling_rights;
        self.ep_sq = prev.ep_sq;
        self.halfmove_clock = prev.halfmove_clock;
        self.key = prev.key;
        self.nnue_pos = prev.nnue_pos;

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

impl position::NNUEPosition {
    fn move_pc(&mut self, from_sq: usize, to_sq: usize) {
        let index = self.board[from_sq];
        self.squares[index] = to_sq;
        self.board.swap(from_sq, to_sq);
    }

    fn remove_pc(&mut self, sq: usize) {
        let index = self.board[sq];
        assert_ne!(index, 32);
        if index != self.end_ptr {
            // Swap captured pc with pc at the end of the array
            let end_sq = self.squares[self.end_ptr];
            self.board[end_sq] = index;
            self.pieces.swap(index, self.end_ptr);
            self.squares.swap(index, self.end_ptr);
        }
        // Wipe info about the pc at the end of array
        self.pieces[self.end_ptr] = 0;
        self.squares[self.end_ptr] = 64;
        self.board[sq] = 32;
        self.end_ptr -= 1;
    }

    fn promote_pc(&mut self, sq: usize, promo_pt: PieceType) {
        let index = self.board[sq];
        assert!(self.pieces[index] == 6 || self.pieces[index] == 12); // Assert it's a pawn (6 or 12)
        self.pieces[index] += promo_pt.to_nnue_pc();
        self.pieces[index] -= 6;
    }
}

#[cfg(test)]
mod nnue_pos_tests {
    use super::*;
    use test_case::test_case;

    #[test_case(STARTPOS; "starting_pos")]
    #[test_case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"; "promote out of check")]
    #[test_case(TPOS2; "test_pos_2")]
    #[test_case(TPOS3; "test_pos_3")]
    #[test_case(TPOS4; "test_pos_4")]
    #[test_case(TPOS5; "test_pos_5")]
    #[test_case(TPOS6; "test_pos_6")]
    fn test_moves(fen: &str) {
        let mut pos = position::Position::from_fen(fen).unwrap();
        let mut movelist = movelist::UnorderedList::new();
        movegen::generate_all(&pos, &mut movelist);

        for mv in movelist.iter() {
            pos.make_move(&mv);
            assert!(
                validate_nnue_pos(&pos),
                "{} {:?}",
                mv.to_algebraic(),
                mv.movetype()
            );
            pos.unmake_move();
        }
    }

    fn validate_nnue_pos(pos: &Position) -> bool {
        let mut i = 0;
        let (w, b) = pos.white_black();

        while pos.nnue_pos.pieces[i] != 0 {
            let pc = pos.nnue_pos.pieces[i];
            let sq = pos.nnue_pos.squares[i];
            // Check board
            if pos.nnue_pos.board[sq] != i {
                return false;
            };

            // Check piece and square info corresponds with bitboards
            let pc_is_white = pc < nnue::Pieces::BKing as usize;
            let expected_pt = if pc_is_white {
                w.pt_at(BB::from_sq(sq))
            } else {
                b.pt_at(BB::from_sq(sq))
            };
            match expected_pt {
                Some(pt) => {
                    let mut expected_pt = pt.to_nnue_pc();
                    if !pc_is_white {
                        expected_pt += 6
                    }
                    if pc != expected_pt {
                        return false;
                    }
                }
                None => panic!("No piece at sq {}", sq),
            }
            i += 1;
            if i == 32 {
                break;
            }
        }
        if pos.occ.pop_count() as usize != i {
            return false;
        }
        let expected_player = if pos.wtm { 0 } else { 1 };
        assert!(expected_player == pos.nnue_pos.player);
        return true;
    }
}
