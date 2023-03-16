/// Implementation of a White / Black state machine to execute turn dependent
/// logic
use super::*;

impl Position {
    /// Reverse the side to move
    pub fn change_state(&mut self) {
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Return the boolean value of whether it's white to move
    pub fn white_to_move(&self) -> bool {
        return matches!(self.side_to_move, Color::White);
    }

    /// Return the BBSet of our piece (side to move)
    pub fn our_pieces(&self) -> &BBSet {
        match self.side_to_move {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    /// Return the BBSet of their pieces (not side to move)
    pub fn their_pieces(&self) -> &BBSet {
        match self.side_to_move {
            Color::White => &self.black,
            Color::Black => &self.white,
        }
    }

    /// Return the rank which pawn promotion occurs
    pub fn target_promotion_rank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_8,
            Color::Black => RANK_1,
        }
    }

    /// Return the rank which pawns promoting originate from
    pub fn src_promotion_rank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_7,
            Color::Black => RANK_2,
        }
    }

    /// Return the rank which pawns start on
    pub fn pawn_start_rank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_2,
            Color::Black => RANK_7,
        }
    }

    /// Return our backrank
    pub fn our_backrank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_1,
            Color::Black => RANK_8,
        }
    }

    /// Return the en passant capture rank mask
    pub fn ep_capture_rank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_5,
            Color::Black => RANK_4,
        }
    }

    /// Return the square on which our kingside rook starts
    pub fn our_ks_rook_starting_sq(&self) -> BB {
        match self.side_to_move {
            Color::White => square::H1,
            Color::Black => square::H8,
        }
    }

    /// Return the square on which our queenside rook starts
    pub fn our_qs_rook_starting_sq(&self) -> BB {
        match self.side_to_move {
            Color::White => square::A1,
            Color::Black => square::A8,
        }
    }

    /// Return the square on which their kingside rook starts
    pub fn their_ks_rook_starting_sq(&self) -> BB {
        match self.side_to_move {
            Color::White => square::H8,
            Color::Black => square::H1,
        }
    }

    // Return the square on which their queenside rook starts
    pub fn their_qs_rook_starting_sq(&self) -> BB {
        match self.side_to_move {
            Color::White => square::A8,
            Color::Black => square::A1,
        }
    }

    /// Translate the provided bitboard in the direction of a single pawn push
    pub fn single_push(&self, src: BB) -> BB {
        match self.side_to_move {
            Color::White => src.north_one(),
            Color::Black => src.south_one(),
        }
    }

    /// Translate the provided bitboard in the direction of a double pawn push
    pub fn double_push(&self, src: BB) -> BB {
        match self.side_to_move {
            Color::White => src.north_two(),
            Color::Black => src.south_two(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn left capture
    pub fn left_capture(&self, src: BB) -> BB {
        match self.side_to_move {
            Color::White => src.nort_west(),
            Color::Black => src.sout_west(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn right capture
    pub fn right_capture(&self, src: BB) -> BB {
        match self.side_to_move {
            Color::White => src.nort_east(),
            Color::Black => src.sout_east(),
        }
    }

    /// Return the pin mask for pawn left captures
    pub fn pawn_left_capture_pin_mask(&self) -> BB {
        match self.side_to_move {
            Color::White => self.white.king.lu_anti_diagonal_mask(),
            Color::Black => self.black.king.lu_diagonal_mask(),
        }
    }

    /// Return the pin mask for pawn right captures
    pub fn pawn_right_capture_pin_mask(&self) -> BB {
        match self.side_to_move {
            Color::White => self.white.king.lu_diagonal_mask(),
            Color::Black => self.black.king.lu_anti_diagonal_mask(),
        }
    }

    /// Return the single push target squares of our pawns
    pub fn pawn_sgl_push_targets(&self) -> BB {
        match self.side_to_move {
            Color::White => self.white.pawn.north_one() & self.free_squares,
            Color::Black => self.black.pawn.south_one() & self.free_squares,
        }
    }

    /// Return the double push target squares of our pawns
    pub fn pawn_dbl_push_targets(&self) -> BB {
        match self.side_to_move {
            Color::White => {
                ((self.white.pawn & RANK_2).north_one() & self.free_squares).north_one()
                    & self.free_squares
            }
            Color::Black => {
                ((self.black.pawn & RANK_7).south_one() & self.free_squares).south_one()
                    & self.free_squares
            }
        }
    }

    /// Return the target rank of double pawn pushes
    pub fn pawn_dbl_push_target_rank(&self) -> BB {
        match self.side_to_move {
            Color::White => RANK_4,
            Color::Black => RANK_5,
        }
    }

    /// Return the left capture target squares of our pawns
    pub fn pawn_lcap_targets(&self) -> BB {
        match self.side_to_move {
            Color::White => self.white.pawn.nort_west() & self.black.all,
            Color::Black => self.black.pawn.sout_west() & self.white.all,
        }
    }

    /// Return the right capture target squares of our pawns
    pub fn pawn_rcap_targets(&self) -> BB {
        match self.side_to_move {
            Color::White => self.white.pawn.nort_east() & self.black.all,
            Color::Black => self.black.pawn.sout_east() & self.white.all,
        }
    }

    /// Return the single push pawn sources from a map of target squares
    pub fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        match self.side_to_move {
            Color::White => targets.south_one(),
            Color::Black => targets.north_one(),
        }
    }

    /// Return the double push pawn sources from a map of target squares
    pub fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        match self.side_to_move {
            Color::White => targets.south_two(),
            Color::Black => targets.north_two(),
        }
    }

    /// Return the left capture pawn sources from a map of target squares
    pub fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        match self.side_to_move {
            Color::White => targets.sout_east(),
            Color::Black => targets.nort_east(),
        }
    }

    /// Return the right capture pawn sources from a map of target squares
    pub fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        match self.side_to_move {
            Color::White => targets.sout_west(),
            Color::Black => targets.nort_west(),
        }
    }

    /// Return the en passant source squares of our pieces
    pub fn pawn_en_passant_srcs(&self) -> BB {
        match self.side_to_move {
            Color::White => {
                (self.en_passant_target_square.sout_east()
                    | self.en_passant_target_square.sout_west())
                    & self.white.pawn
            }
            Color::Black => {
                (self.en_passant_target_square.nort_east()
                    | self.en_passant_target_square.nort_west())
                    & self.black.pawn
            }
        }
    }

    /// Return the square of the piece being captured by en passant
    pub fn pawn_en_passant_capture_square(&self) -> BB {
        match self.side_to_move {
            Color::White => self.en_passant_target_square.south_one(),
            Color::Black => self.en_passant_target_square.north_one(),
        }
    }

    /// Return the mask of the squares the king must traverse to castle kingside
    pub fn kingside_castle_mask(&self) -> BB {
        const WHITE: BB = BB(square::F1.0 | square::G1.0);
        const BLACK: BB = BB(square::F8.0 | square::G8.0);
        match self.side_to_move {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares the king must traverse to castle
    /// queenside so must be safe
    pub fn queenside_castle_safety_mask(&self) -> BB {
        const WHITE: BB = BB(square::C1.0 | square::D1.0);
        const BLACK: BB = BB(square::C8.0 | square::D8.0);
        match self.side_to_move {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares in between the king and the rook which
    /// must be free in order to castle
    pub fn queenside_castle_free_mask(&self) -> BB {
        const WHITE: BB = BB(square::B1.0 | square::C1.0 | square::D1.0);
        const BLACK: BB = BB(square::B8.0 | square::C8.0 | square::D8.0);
        match self.side_to_move {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return our king side castling rights
    pub fn our_kingside_castle(&self) -> bool {
        match self.side_to_move {
            Color::White => (self.castling_rights & square::H1).is_not_empty(),
            Color::Black => (self.castling_rights & square::H8).is_not_empty(),
        }
    }

    /// Return the queenside castling rights
    pub fn our_queenside_castle(&self) -> bool {
        match self.side_to_move {
            Color::White => (self.castling_rights & square::A1).is_not_empty(),
            Color::Black => (self.castling_rights & square::A8).is_not_empty(),
        }
    }

    /// Return all the squares attacked by their pawns
    pub fn unsafe_squares_pawn(&self) -> BB {
        match self.side_to_move {
            Color::White => self.black.pawn.sout_east() | self.black.pawn.sout_west(),
            Color::Black => self.white.pawn.nort_east() | self.white.pawn.nort_west(),
        }
    }

    /// Locate their pawns checking our king
    pub fn their_checking_pawns(&self) -> BB {
        match self.side_to_move {
            Color::White => {
                (self.white.king.nort_west() | self.white.king.nort_east()) & self.black.pawn
            }
            Color::Black => {
                (self.black.king.sout_west() | self.black.king.sout_east()) & self.white.pawn
            }
        }
    }

    /// Return our piece set as a mutable reference
    pub fn mut_our_pieces(&mut self) -> &mut BBSet {
        match self.side_to_move {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    /// Return their piece set as a mutable reference
    pub fn mut_their_pieces(&mut self) -> &mut BBSet {
        match self.side_to_move {
            Color::White => &mut self.black,
            Color::Black => &mut self.white,
        }
    }
}
