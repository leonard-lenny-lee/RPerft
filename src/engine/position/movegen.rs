use super::*;
use movelist::MoveList;
use position::Position;

impl Position {

    /// Generate a MoveList of all legal moves
    pub fn find_moves(&self) -> MoveList {
        // Initialise variables
        let mut move_list: MoveList = MoveList::new();
        let unsafe_squares = self.unsafe_squares();
        let checkers = self.find_checkers();
        let pinned_pieces = self.pinned_pieces();
        // Number of pieces placing the king in check
        let n_checkers = checkers.pop_count();
        let mut capture_mask = FILLED_BB;
        let mut push_mask = FILLED_BB;
        // If the king is in double check, only king moves are valid
        if n_checkers > 1 {
            self.find_king_moves(&mut move_list, unsafe_squares);
            return move_list;
        }
        if n_checkers == 1 {
            // In single check, the attacking piece can also be captured so set
            // the location of the checker as the capture mask
            capture_mask = checkers;
            if self.their_piece_at_is_slider(checkers) {
                // If the attacker is a sliding piece, then check can be 
                // blocked by another piece moving to the intervening squares
                push_mask = self.our_pieces().king.connect_squares(checkers)
            } else {
                // It can only be captured so give no options to block
                push_mask = EMPTY_BB
            }
        }

        // Add all moves to the move list
        self.find_single_pushes(&mut move_list, push_mask, pinned_pieces);
        self.find_double_pushes(&mut move_list, push_mask, pinned_pieces);
        self.find_right_captures(&mut move_list, capture_mask, pinned_pieces);
        self.find_left_captures(&mut move_list, capture_mask, pinned_pieces);
        self.find_knight_moves(&mut move_list, capture_mask, push_mask, pinned_pieces);
        self.find_king_moves(&mut move_list, unsafe_squares);
        self.find_bishop_moves(&mut move_list, capture_mask, push_mask, pinned_pieces);
        self.find_rook_moves(&mut move_list, capture_mask, push_mask, pinned_pieces);
        self.find_queen_moves(&mut move_list, capture_mask, push_mask, pinned_pieces);
        // Castling is only allowed if not in check
        if n_checkers == 0 {
            self.find_castling_moves(&mut move_list, unsafe_squares);
        }
        self.find_en_passant_moves(
            &mut move_list, capture_mask, push_mask, pinned_pieces
        );
        return move_list;
    }

    // Move generation methods to push valid moves onto the move list

    /// Move generation function to find all pawn single pushes in a position.
    fn find_single_pushes(
        &self, move_list: &mut MoveList, push_mask: BB, pinned_pieces: BB
    ) {
        let targets = self.pawn_sgl_push_targets() & push_mask;
        let srcs = self.pawn_sgl_push_srcs(targets);
        // Separate promoting pawns from non-promoting pawns
        let mut promotion_pawns = srcs & self.promotion_rank();
        let mut normal_pawns = srcs ^ promotion_pawns;
        // Separate pinned pawns
        let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
        promotion_pawns ^= pinned_promotion_pawns;
        let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
        normal_pawns ^= pinned_normal_pawns;
        
        // Normal pawns
        while normal_pawns.is_any() {
            let src = normal_pawns.pop_ls1b();
            let target = self.pawn_sgl_push(src);
            move_list.add_quiet_move(target, src);
        }
        // Promotion pawns
        while promotion_pawns.is_any() {
            let src = promotion_pawns.pop_ls1b();
            let target = self.pawn_sgl_push(src);
            move_list.add_promotions(target, src);
        }
        // For pinned pieces, only allow moves towards / away from the king
        while pinned_normal_pawns.is_any() {
            let src = pinned_normal_pawns.pop_ls1b();
            let mut target = self.pawn_sgl_push(src);
            // Only allow targets that are on the king-piece axis
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_quiet_move(target, src);
            }
        }
        while pinned_promotion_pawns.is_any() {
            let src = pinned_promotion_pawns.pop_ls1b();
            let mut target = self.pawn_sgl_push(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_promotions(target, src);
            }
        }

    }

    /// Move generation function to find all pawn double pushes in a position
    fn find_double_pushes(
        &self, move_list: &mut MoveList, push_mask: BB, pinned_pieces: BB
    ) {
        let targets = self.pawn_dbl_push_targets() & push_mask;
        let mut srcs = self.pawn_dbl_push_srcs(targets);
        let mut pinned_srcs = srcs & pinned_pieces;
        srcs ^= pinned_srcs;
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            let target = self.pawn_dbl_push(src);    
            move_list.add_double_pawn_push(target, src);
        }
        // For pinned pieces, only allow moves towards / away from the king
        while pinned_srcs.is_any() {
            let src = pinned_srcs.pop_ls1b();
            let mut target = self.pawn_dbl_push(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_double_pawn_push(target, src);
            }
        }
    }

    /// Move generation function to find all pawn left captures in a position
    fn find_left_captures(
        &self, move_list: &mut MoveList, capture_mask: BB, pinned_pieces: BB
    ) {
        let targets = self.pawn_lcap_targets() & capture_mask;
        let srcs = self.pawn_lcap_srcs(targets);
        // Separate promotion pawns from non-promoting pawns
        let mut promotion_pawns = srcs & self.promotion_rank();
        let mut normal_pawns = srcs ^ promotion_pawns;
        // Separate pinned pawns
        let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
        promotion_pawns ^= pinned_promotion_pawns;
        let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
        normal_pawns ^= pinned_normal_pawns;

        // Normal pawns
        while normal_pawns.is_any() {
            let src = normal_pawns.pop_ls1b();
            let target = self.pawn_left_capture(src);
            move_list.add_capture(target, src);
        }
        // Promotion pawns
        while promotion_pawns.is_any() {
            let src = promotion_pawns.pop_ls1b();
            let target = self.pawn_left_capture(src);
            move_list.add_promotion_captures(target, src);
        }
        // For pinned pieces, only allow moves towards / away from the king
        while pinned_normal_pawns.is_any() {
            let src = pinned_normal_pawns.pop_ls1b();
            let mut target = self.pawn_left_capture(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_capture(target, src);
            }
        }
        while pinned_promotion_pawns.is_any() {
            let src = pinned_promotion_pawns.pop_ls1b();
            let mut target = self.pawn_left_capture(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_promotion_captures(target, src);
            }
        }
    }

    /// Move generation function to find all pawn right captures in a position
    fn find_right_captures(
        &self, move_list: &mut MoveList, capture_mask: BB, pinned_pieces: BB
    ) {
        let targets = self.pawn_rcap_targets() & capture_mask;
        let srcs = self.pawn_rcap_srcs(targets);
        // Separate promotion pawns from non-promoting pawns
        let mut promotion_pawns = srcs & self.promotion_rank();
        let mut normal_pawns = srcs ^ promotion_pawns;
        // Separate pinned pawns
        let mut pinned_promotion_pawns = promotion_pawns & pinned_pieces;
        promotion_pawns ^= pinned_promotion_pawns;
        let mut pinned_normal_pawns = normal_pawns & pinned_pieces;
        normal_pawns ^= pinned_normal_pawns;

        // Normal pawns
        while normal_pawns.is_any() {
            let src = normal_pawns.pop_ls1b();
            let target = self.pawn_right_capture(src);
            move_list.add_capture(target, src);
        }
        // Promotion pawns
        while promotion_pawns.is_any() {
            let src = promotion_pawns.pop_ls1b();
            let target = self.pawn_right_capture(src);
            move_list.add_promotion_captures(target, src);
        }
        // For pinned pieces, only allow moves towards / away from the king
        while pinned_normal_pawns.is_any() {
            let src = pinned_normal_pawns.pop_ls1b();
            let mut target = self.pawn_right_capture(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_capture(target, src);
            }
        }
        while pinned_promotion_pawns.is_any() {
            let src = pinned_promotion_pawns.pop_ls1b();
            let mut target = self.pawn_right_capture(src);
            target &= self.our_pieces().king.common_axis(src);
            if target.is_any() {
                move_list.add_promotion_captures(target, src);
            }
        }
    }

    /// Move generation function for knights
    fn find_knight_moves(
        &self, move_list: &mut MoveList, capture_mask: BB,
        push_mask: BB, pinned_pieces: BB,
    ) {
        let our_pieces = self.our_pieces();
        let mut srcs = our_pieces.knight;
        // Filter out knights which are pinned - pinned knights have no legal moves
        srcs &= !pinned_pieces;
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            let mut targets = src.lookup_knight_attacks() & !our_pieces.any;
            // Only allow moves which either capture a checking piece or blocks
            // the check. These masks should be a FILLED_BB when no check.
            targets &= capture_mask | push_mask;
            self.find_quiet_moves_and_captures(move_list, targets, src)
        }
    }

    /// Append onto a move list the king moves
    fn find_king_moves(&self, move_list: &mut MoveList, unsafe_squares: BB) {
        let our_pieces = self.our_pieces();
        let src = our_pieces.king;
        let mut targets = src.lookup_king_attacks() & !our_pieces.any;
        // Remove unsafe squares i.e. squares attacked by opponent pieces
        // from the available target sqaures for the king
        targets &= !unsafe_squares;
        self.find_quiet_moves_and_captures(move_list, targets, src)
    }

    fn find_bishop_moves(
        &self, move_list: &mut MoveList, capture_mask: BB,
        push_mask: BB, pinned_pieces: BB,
    ) {
        let our_pieces = self.our_pieces();
        let mut srcs = our_pieces.bishop;
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            let mut targets: BB = src.lookup_bishop_attacks(self.data.occ);
            targets &= !our_pieces.any;
            targets &= capture_mask | push_mask;
            if (src & pinned_pieces).is_any() {
                targets &= our_pieces.king.common_axis(src);
            }
            self.find_quiet_moves_and_captures(move_list, targets, src)
        }
    }

    fn find_rook_moves(
        &self, move_list: &mut MoveList, capture_mask: BB,
        push_mask: BB, pinned_pieces: BB,
    ) {
        let our_pieces = self.our_pieces();
        let mut srcs = our_pieces.rook;
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            let mut targets: BB = src.lookup_rook_attacks(self.data.occ);
            targets &= !our_pieces.any;
            targets &= capture_mask | push_mask;
            if (src & pinned_pieces).is_any() {
                targets &= our_pieces.king.common_axis(src);
            }
            self.find_quiet_moves_and_captures(move_list, targets, src)
        }
    }

    fn find_queen_moves(
        &self, move_list: &mut MoveList, capture_mask: BB,
        push_mask: BB, pinned_pieces: BB,
    ) {
        let our_pieces = self.our_pieces();
        let mut srcs = our_pieces.queen;
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            let mut targets: BB = src.lookup_queen_attacks(self.data.occ);
            targets &= !our_pieces.any;
            targets &= capture_mask | push_mask;
            if (src & pinned_pieces).is_any() {
                targets &= our_pieces.king.common_axis(src);
            }
            self.find_quiet_moves_and_captures(move_list, targets, src)
        }
    }

    // Special Moves

    /// Move generation function for en passant captures
    fn find_en_passant_moves(
        &self, move_list: &mut MoveList, capture_mask: BB,
        push_mask: BB, pinned_pieces: BB
    ) {
        let target = self.data.en_passant_target_sq;
        let captured_pawn = self.pawn_en_passant_cap();
        if target == EMPTY_BB || (captured_pawn & capture_mask == EMPTY_BB
            && target & push_mask == EMPTY_BB) {
            return
        }
        let our_pieces = self.our_pieces();
        let their_pieces = self.their_pieces();
        let mut srcs = self.pawn_en_passant_srcs();
        
        while srcs.is_any() {
            let src = srcs.pop_ls1b();
            // If pawn is pinned, check capture is along pin axis
            if (src & pinned_pieces).is_any() {
                let pin_mask = our_pieces.king.common_axis(src);
                if target & pin_mask == EMPTY_BB {
                    continue;
                }
            }
            // Check rare en passant case that may occur if the king is on the
            // same rank as the pawns involved in the en passant capture where
            // an en passant capture may reveal a discovered check
            if (our_pieces.king & self.ep_capture_rank()).is_any() {
                let occ = self.data.occ & !(src | captured_pawn);
                if (our_pieces.king.hyp_quint(occ, Axis::Rank)
                    & (their_pieces.rook | their_pieces.queen)).is_any() {
                    continue;
                }
            }
            move_list.add_en_passant_capture(target, src);
        }
        
    }

    fn find_castling_moves(
        &self, move_list: &mut MoveList, unsafe_squares: BB
    ) {
        let src = self.our_pieces().king;
        // Kingside castle
        if self.our_kingside_castle()
            && (self.kingside_castle_mask() & self.data.occ).is_empty()
            && (self.kingside_castle_mask() & unsafe_squares).is_empty()
        {
            move_list.add_short_castle(src.east_two(), src)
        }
        // Queenside castle
        if self.our_queenside_castle()
            && (self.queenside_castle_mask_free() & self.data.occ).is_empty()
            && (self.queenside_castle_mask_safe() & unsafe_squares).is_empty()
        {
            move_list.add_long_castle(src.west_two(), src)
        }
    }

    /// Helper function for non-pawn moves, where the capture status is
    /// indeterminate. Seperates out the capture moves from the quiet moves and
    /// adds them to the move vector
    fn find_quiet_moves_and_captures(
        &self, move_list: &mut MoveList, targets: BB, src: BB
    ) {
        let mut capture_targets = targets & self.their_pieces().any;
        let mut quiet_targets = targets & self.data.free;
        while capture_targets.is_any() {
            let target = capture_targets.pop_ls1b();
            move_list.add_capture(target, src);
        }
        while quiet_targets.is_any() {
            let target = quiet_targets.pop_ls1b();
            move_list.add_quiet_move(target, src);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    fn generate_targets(move_list: MoveList) -> BB {
        let mut targets = EMPTY_BB;
        for mv in move_list.iter() {
            targets |= mv.target();
        }
        return targets
    }

    #[test_case(DEFAULT_FEN, 8, vec![16, 17, 18, 19, 20, 21, 22, 23]; "starting")]
    #[test_case(POSITION_2, 4, vec![16, 17, 43, 22]; "position_two")]
    #[test_case(POSITION_3, 3, vec![20, 22, 41]; "position_three")]
    fn test_sgl_push_pawn_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_single_pushes( &mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets);
    }

    #[test_case(DEFAULT_FEN, 8, vec![24, 25, 26, 27, 28, 29, 30, 31]; "starting")]
    #[test_case(POSITION_2, 2, vec![24, 30]; "position_two")]
    #[test_case(POSITION_3, 2, vec![28, 30]; "position_three")]
    fn test_dbl_push_pawn_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_double_pushes(&mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 0, vec![]; "position_two")]
    #[test_case(POSITION_3, 0, vec![]; "position_three")]
    fn test_push_lcap_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_left_captures(&mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![44, 23]; "position_two")]
    #[test_case(POSITION_3, 0, vec![]; "position_three")]
    fn test_push_rcap_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_right_captures(&mut move_list, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }
    #[test_case(DEFAULT_FEN, 4, vec![16, 18, 21, 23]; "starting")]
    #[test_case(POSITION_2, 11, vec![1, 24, 33, 3, 51, 42, 26, 19, 30, 46, 53];
        "position_two")]
    fn test_knight_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_knight_moves(&mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![3, 5]; "position_two")]
    fn test_king_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_king_moves(&mut move_list, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 11, vec![2, 20, 29, 38, 47, 3, 5, 19, 26, 33, 40];
        "position_two")]
    fn test_bishop_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_bishop_moves(&mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 5, vec![1, 2, 3, 5, 6]; "position_two")]
    fn test_rook_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_rook_moves(&mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 9, vec![19, 20, 22, 23, 29, 37, 45, 30, 39];
        "position_two")]
    fn test_queen_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_queen_moves( &mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 0, vec![]; "position_two")]
    fn test_en_passant_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_en_passant_moves( &mut move_list, FILLED_BB, FILLED_BB, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 0, vec![]; "starting")]
    #[test_case(POSITION_2, 2, vec![2, 6]; "position_two")]
    fn test_castling_move_gen(
        fen: &str, expected_nodes: i32, expected_targets: Vec<usize>
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let mut move_list = MoveList::new();
        pos.find_castling_moves(&mut move_list, EMPTY_BB);
        assert_eq!(expected_nodes, move_list.len() as i32);
        let targets = generate_targets(move_list);
        let expected_targets = BB::from_indices(expected_targets);
        assert_eq!(expected_targets, targets)
    }

    #[test_case(DEFAULT_FEN, 20, 0; "starting")]
    #[test_case(POSITION_2, 48, 8; "position_two")]
    #[test_case(POSITION_3, 14, 1; "position_three")]
    #[test_case(POSITION_4, 6, 0; "position_four")]
    fn test_move_gen(
        fen: &str, expected_nodes: i32, expected_captures: i32,
    ) {
        let pos = Position::from_fen(fen.to_string()).unwrap();
        let move_list = pos.find_moves();
        let mut n_captures = 0;
        for mv in move_list.iter() {
            if mv.is_capture() {
                n_captures += 1
            }
        }
        assert_eq!(expected_nodes, move_list.len() as i32, "nodes");
        assert_eq!(expected_captures, n_captures, "captures")
    }
}