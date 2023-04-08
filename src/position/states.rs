/// Implementation of a White / Black state machine to execute turn dependent
/// logic
use super::*;

impl Position {
    /// Reverse the side to move
    pub fn change_state(&mut self) {
        self.stm = match self.stm {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        self.wtm = !self.wtm;
        std::mem::swap(&mut self.us, &mut self.them)
    }

    pub fn white_black(&self) -> (&BBSet, &BBSet) {
        match self.stm {
            Color::White => (&self.us, &self.them),
            Color::Black => (&self.them, &self.us),
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
    pub fn ksr_start(&self) -> BB {
        match self.stm {
            Color::White => square::H1,
            Color::Black => square::H8,
        }
    }

    /// Return the square on which our queenside rook starts
    pub fn qsr_start(&self) -> BB {
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

    /// Return the axis on which left capture pins occur
    pub fn lcap_axis(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.adiag(),
            Color::Black => bb.diag(),
        }
    }

    /// Return the axis on which right capture pins occur
    pub fn rcap_axis(&self, bb: BB) -> BB {
        match self.stm {
            Color::White => bb.diag(),
            Color::Black => bb.adiag(),
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
    pub fn qsc_safe_mask(&self) -> BB {
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
}
