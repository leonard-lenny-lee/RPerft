/// Make move function for applying a move to a position
use super::*;
use movelist::Move;
use types::{
    MoveType::*,
    PieceType::{self, *},
};

impl position::Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&mut self, mv: &Move) {
        // Unpack move data
        let to = mv.to();
        let from = mv.from();
        let mt = mv.movetype();
        let captured_pt = self.them.piecetype_at(to);
        let moved_pt = self.us.piecetype_at(from).expect("from must be occ");

        // Push data to stack
        let mut stack_data = position::StackData {
            from,
            to,
            mt,
            moved_pt,
            captured_pt,
            castling_rights: self.castling_rights,
            halfmove_clock: self.halfmove_clock,
            en_passant: self.en_passant,
            key: self.key,
            restore_index: None,
        };

        // NNUE routine
        let from_sq = from.to_square();
        let to_sq = to.to_square();
        self.nnue_pos.move_pc(from_sq, to_sq);

        // Undo current ep key before position is modified
        self.en_passant_key_update();

        // Increment clocks
        self.halfmove_clock += 1;
        self.fullmove_clock += self.side_to_move as u8;

        // Source squares must be free and target squares must be occupied
        self.free |= from;
        self.free &= !to;

        // Our bitboards must be flipped at the target and source
        let move_mask = from | to;
        self.us[moved_pt] ^= move_mask;
        self.us.all ^= move_mask;
        self.move_key_update(moved_pt, from, to, self.white_to_move);

        // Reset halfmove clock on pawn moves, remove castle rights on king moves
        match moved_pt {
            Pawn => self.halfmove_clock = 0,
            King => self.castling_rights &= !self.rank_1(),
            _ => (),
        }

        // If the rooks have moved, remove right to castle on that side
        self.castling_rights &= !from;

        // Set ep target to empty, set later if dbl pawn push
        self.en_passant = constants::bb::EMPTY;

        // Captures, excluding en passant
        if let Some(pt) = captured_pt {
            self.them[pt] ^= to;
            self.them.all ^= to;
            self.square_key_update(pt, to, !self.white_to_move);
            // Remove castling right if rook has been captured
            self.castling_rights &= !to;
            self.halfmove_clock = 0;
            // Index swap has already occurred so from_sq actually points to the target pc
            stack_data.restore_index = Some(self.nnue_pos.remove_pc(from_sq));
        }

        // Promotions
        if mv.is_promotion() {
            let promo_pt = mv.promotion_piecetype().expect("must encode pt");
            self.us[promo_pt] ^= to;
            self.us.pawn ^= to;
            self.square_key_update(Pawn, to, self.white_to_move);
            self.square_key_update(promo_pt, to, self.white_to_move);
            self.nnue_pos.mutate_pc(to_sq, promo_pt);
        }

        // Execute special actions
        match mt {
            DoublePawnPush => {
                // Ep target is one square behind dbl push target
                self.en_passant = self.back_one(to);
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
                self.move_key_update(Rook, rook_from, rook_to, self.white_to_move);
                self.nnue_pos
                    .move_pc(rook_from.to_square(), rook_to.to_square());
            }

            EnPassant => {
                let ep_sq = self.back_one(to);
                self.them.pawn ^= ep_sq;
                self.them.all ^= ep_sq;
                self.free ^= ep_sq;
                self.square_key_update(Pawn, ep_sq, !self.white_to_move);
                stack_data.restore_index = Some(self.nnue_pos.remove_pc(ep_sq.to_square()));
            }

            _ => (),
        }

        self.occupied = !self.free;
        // Change the turn and state
        self.nnue_pos.player ^= 1;
        self.change_state();
        self.push_to_stack(stack_data);
        // Update key
        self.turn_key_update();
        self.en_passant_key_update();
        self.castling_key_update();
    }

    pub fn unmake_move(&mut self) {
        let prev = self.pop_from_stack();

        // Reverse clocks
        self.halfmove_clock = prev.halfmove_clock;
        self.fullmove_clock -= 1 - self.side_to_move as u8;

        // Restore info from the stack
        self.castling_rights = prev.castling_rights;
        self.en_passant = prev.en_passant;
        self.halfmove_clock = prev.halfmove_clock;
        self.key = prev.key;

        // Source square must now be occupied, free target square for now
        self.free |= prev.to;
        self.free &= !prev.from;

        // Reverse changes to our bbs
        let moved_pt = self.them.piecetype_at(prev.to).expect("must be occupied");
        let move_mask = prev.from | prev.to;
        self.them.all ^= move_mask;

        // NNUE position
        self.nnue_pos.player ^= 1;
        let from_sq = prev.from.to_square();
        let to_sq = prev.to.to_square();
        self.nnue_pos.move_pc(to_sq, from_sq);

        // Undo capture
        if let Some(pt) = prev.captured_pt {
            self.us[pt] ^= prev.to;
            self.us.all ^= prev.to;
            self.free ^= prev.to;
            self.nnue_pos
                .replace_pc(to_sq, pt, prev.restore_index.unwrap())
        }

        if moved_pt == prev.moved_pt {
            self.them[moved_pt] ^= move_mask;
        } else {
            // Must be a promotion
            self.them[moved_pt] ^= prev.to;
            self.them.pawn ^= prev.from;
            self.nnue_pos.mutate_pc(from_sq, PieceType::Pawn);
        }

        // Undo special actions
        match prev.mt {
            ShortCastle | LongCastle => {
                let (rook_from, rook_to) = if let ShortCastle = prev.mt {
                    (prev.to.east_one(), prev.to.west_one())
                } else {
                    (prev.to.west_two(), prev.to.east_one())
                };
                let mask = rook_from | rook_to;
                self.them.rook ^= mask;
                self.them.all ^= mask;
                self.free ^= mask;
                self.nnue_pos
                    .move_pc(rook_to.to_square(), rook_from.to_square())
            }

            EnPassant => {
                let ep_sq = self.forward_one(prev.to);
                self.us.pawn ^= ep_sq;
                self.us.all ^= ep_sq;
                self.free ^= ep_sq;
                self.nnue_pos.replace_pc(
                    ep_sq.to_square(),
                    PieceType::Pawn,
                    prev.restore_index.unwrap(),
                )
            }

            _ => (),
        }

        self.occupied = !self.free;
        self.change_state();
    }
}

impl position::NNUEPosition {
    fn move_pc(&mut self, from_sq: usize, to_sq: usize) {
        let index = self.board[from_sq];
        self.squares[index] = to_sq;
        self.board.swap(from_sq, to_sq);
    }

    fn remove_pc(&mut self, sq: usize) -> usize {
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
        return index;
    }

    fn replace_pc(&mut self, sq: usize, pt: PieceType, restore_index: usize) {
        self.end_ptr += 1;
        self.pieces[self.end_ptr] = pt.to_nnue_pc() + (self.player ^ 1) * 6;
        self.squares[self.end_ptr] = sq;
        self.board[sq] = self.end_ptr;
        if restore_index != self.end_ptr {
            let prev_sq = self.squares[restore_index];
            self.pieces.swap(restore_index, self.end_ptr);
            self.squares.swap(restore_index, self.end_ptr);
            self.board.swap(prev_sq, sq);
        }
    }

    fn mutate_pc(&mut self, sq: usize, pt: PieceType) {
        let index = self.board[sq];
        let cur_pt = self.pieces[index] - self.player * 6;
        self.pieces[index] += pt.to_nnue_pc();
        self.pieces[index] -= cur_pt;
    }
}

impl position::Position {
    fn push_to_stack(&mut self, mut stack_data: position::StackData) {
        let entry = &mut self.stack[self.ply as usize];
        std::mem::swap(entry, &mut stack_data);
        self.ply += 1;
    }

    fn pop_from_stack(&mut self) -> position::StackData {
        self.ply -= 1;
        let mut stack_data = position::StackData::default();
        let stack_entry = &mut self.stack[self.ply as usize];
        std::mem::swap(&mut stack_data, stack_entry);
        stack_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::fen::*;
    use test_case::test_case;

    #[test_case(START; "starting_pos")]
    #[test_case(TEST_2; "test_pos_2")]
    #[test_case(TEST_3; "test_pos_3")]
    #[test_case(TEST_4; "test_pos_4")]
    #[test_case(TEST_5; "test_pos_5")]
    #[test_case(TEST_6; "test_pos_6")]
    #[test_case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1"; "illegal ep move #1")]
    #[test_case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1"; "illegal ep move #2")]
    #[test_case("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1"; "ep capture checks opponent")]
    #[test_case("5k2/8/8/8/8/8/8/4K2R w K - 0 1"; "short castling gives check")]
    #[test_case("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1"; "long castling gives check")]
    #[test_case("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1"; "castle rights")]
    #[test_case("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1"; "castling prevented")]
    #[test_case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"; "promote out of check")]
    #[test_case("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1"; "discovered check")]
    #[test_case("4k3/1P6/8/8/8/8/K7/8 w - - 0 1"; "promote to give check")]
    #[test_case("8/P1k5/K7/8/8/8/8/8 w - - 0 1"; "under promote to give check")]
    #[test_case("K1k5/8/P7/8/8/8/8/8 w - - 0 1"; "self statemate")]
    #[test_case("8/k1P5/8/1K6/8/8/8/8 w - - 0 1"; "stalemate & checkmate")]
    #[test_case("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1"; "stalemate & checkmate #2")]
    fn test_nnue_pos_make_unmake(fen: &str) {
        let mut position = position::Position::from_fen(fen).unwrap();
        let mut movelist = movelist::UnorderedList::new();
        movegen::generate_all(&position, &mut movelist);

        for mv in movelist.iter() {
            position.make_move(&mv);
            // Check make
            assert!(
                validate_nnue_pos(&position),
                "makeerror={} {:?}",
                mv.to_algebraic(),
                mv.movetype()
            );
            position.unmake_move();
            // Check unmake
            assert!(
                validate_nnue_pos(&position),
                "unmakeerror={} {} {:?}",
                position.to_fen(),
                mv.to_algebraic(),
                mv.movetype()
            );
        }
    }

    fn validate_nnue_pos(position: &position::Position) -> bool {
        let mut i = 0;
        let (white, black) = position.white_black_bitboards();

        while position.nnue_pos.pieces[i] != 0 {
            let pc = position.nnue_pos.pieces[i];
            let sq = position.nnue_pos.squares[i];
            // Check board
            if position.nnue_pos.board[sq] != i {
                return false;
            };

            // Check piece and square info corresponds with bitboards
            let pc_is_white = pc < nnue::Pieces::BKing as usize;
            let expected_pt = if pc_is_white {
                white.piecetype_at(BitBoard::from_square(sq))
            } else {
                black.piecetype_at(BitBoard::from_square(sq))
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
                None => return false,
            }
            i += 1;
            if i == 32 {
                break;
            }
        }
        if position.occupied.pop_count() as usize != i {
            return false;
        }

        // Final check
        let expected_player = if position.white_to_move { 0 } else { 1 };
        return expected_player == position.nnue_pos.player;
    }
}
