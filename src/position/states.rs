/// Implementation of a White / Black state machine to execute turn dependent
/// logic
use super::*;

impl Position {
    /// Reverse the side to move
    pub fn change_state(&mut self) {
        self.stm = match self.stm {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Return the boolean value of whether it's white to move
    pub fn wtm(&self) -> bool {
        return matches!(self.stm, Color::White);
    }

    pub fn us_them(&self) -> (&BBSet, &BBSet) {
        match self.stm {
            Color::White => (&self.white, &self.black),
            Color::Black => (&self.black, &self.white),
        }
    }

    /// Return the BBSet of our piece (side to move)
    pub fn us(&self) -> &BBSet {
        match self.stm {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    /// Return the BBSet of their pieces (not side to move)
    pub fn them(&self) -> &BBSet {
        match self.stm {
            Color::White => &self.black,
            Color::Black => &self.white,
        }
    }

    /// Return the rank which pawn promotion occurs
    pub fn rank_8(&self) -> BB {
        match self.stm {
            Color::White => RANK_8,
            Color::Black => RANK_1,
        }
    }

    /// Return the rank which pawns promoting originate from
    pub fn rank_7(&self) -> BB {
        match self.stm {
            Color::White => RANK_7,
            Color::Black => RANK_2,
        }
    }

    /// Return the rank which pawns start on
    pub fn rank_2(&self) -> BB {
        match self.stm {
            Color::White => RANK_2,
            Color::Black => RANK_7,
        }
    }

    pub fn rank_3(&self) -> BB {
        match self.stm {
            Color::White => RANK_3,
            Color::Black => RANK_6,
        }
    }

    /// Return our backrank
    pub fn rank_1(&self) -> BB {
        match self.stm {
            Color::White => RANK_1,
            Color::Black => RANK_8,
        }
    }

    /// Return the en passant capture rank mask
    pub fn rank_5(&self) -> BB {
        match self.stm {
            Color::White => RANK_5,
            Color::Black => RANK_4,
        }
    }

    /// Return the square on which our kingside rook starts
    pub fn our_ksr_start(&self) -> BB {
        match self.stm {
            Color::White => square::H1,
            Color::Black => square::H8,
        }
    }

    /// Return the square on which our queenside rook starts
    pub fn our_qsr_start(&self) -> BB {
        match self.stm {
            Color::White => square::A1,
            Color::Black => square::A8,
        }
    }

    /// Translate the provided bitboard in the direction of a single pawn push
    pub fn push(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.north_one(),
            Color::Black => bb.south_one(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn left capture
    pub fn lcap(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.nort_west(),
            Color::Black => bb.sout_west(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn right capture
    pub fn rcap(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.nort_east(),
            Color::Black => bb.sout_east(),
        }
    }

    /// Return a BB pushed back one square towards backrank
    pub fn push_back(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.south_one(),
            Color::Black => bb.north_one(),
        }
    }

    /// Return a BB pushed back two squares towards the backrank
    pub fn push_back_two(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.south_two(),
            Color::Black => bb.north_two(),
        }
    }

    /// Return a BB pushed back one square opposite the left capture direction
    pub fn lcap_back(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.sout_east(),
            Color::Black => bb.nort_east(),
        }
    }

    /// Return the right capture pawn sources from a map of target squares
    pub fn rcap_back(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.sout_west(),
            Color::Black => bb.nort_west(),
        }
    }

    /// Return the mask of the squares the king must traverse to castle kingside
    pub fn ksc_mask(&self) -> BB {
        const WHITE: BB = BB(square::F1.0 | square::G1.0);
        const BLACK: BB = BB(square::F8.0 | square::G8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares the king must traverse to castle
    /// queenside so must be safe
    pub fn qsc_mask(&self) -> BB {
        const WHITE: BB = BB(square::C1.0 | square::D1.0);
        const BLACK: BB = BB(square::C8.0 | square::D8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares in between the king and the rook which
    /// must be free in order to castle
    pub fn qsc_free_mask(&self) -> BB {
        const WHITE: BB = BB(square::B1.0 | square::C1.0 | square::D1.0);
        const BLACK: BB = BB(square::B8.0 | square::C8.0 | square::D8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return our king side castling rights
    pub fn can_ksc(&self) -> bool {
        match self.stm {
            Color::White => (self.castling_rights & square::H1).is_not_empty(),
            Color::Black => (self.castling_rights & square::H8).is_not_empty(),
        }
    }

    /// Return the queenside castling rights
    pub fn can_qsc(&self) -> bool {
        match self.stm {
            Color::White => (self.castling_rights & square::A1).is_not_empty(),
            Color::Black => (self.castling_rights & square::A8).is_not_empty(),
        }
    }

    /// Return our piece set as a mutable reference
    pub fn mut_us(&mut self) -> &mut BBSet {
        match self.stm {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    /// Return their piece set as a mutable reference
    pub fn mut_them(&mut self) -> &mut BBSet {
        match self.stm {
            Color::White => &mut self.black,
            Color::Black => &mut self.white,
        }
    }
}
