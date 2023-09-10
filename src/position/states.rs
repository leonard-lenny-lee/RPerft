/// Implementation of a White / Black state machine to execute turn dependent logic
use super::*;
use constants::bb;
use constants::rank::*;

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

    pub fn white_black(&self) -> (&BitBoardSet, &BitBoardSet) {
        match self.stm {
            Color::White => (&self.us, &self.them),
            Color::Black => (&self.them, &self.us),
        }
    }

    /// Return the rank which pawns promoting originate from
    pub fn rank_7(&self) -> BitBoard {
        match self.stm {
            Color::White => RANK_7,
            Color::Black => RANK_2,
        }
    }

    /// Return the rank which pawns start on
    pub fn rank_2(&self) -> BitBoard {
        match self.stm {
            Color::White => RANK_2,
            Color::Black => RANK_7,
        }
    }

    pub fn rank_3(&self) -> BitBoard {
        match self.stm {
            Color::White => RANK_3,
            Color::Black => RANK_6,
        }
    }

    /// Return our backrank
    pub fn rank_1(&self) -> BitBoard {
        match self.stm {
            Color::White => RANK_1,
            Color::Black => RANK_8,
        }
    }

    /// Return the en passant capture rank mask
    pub fn rank_5(&self) -> BitBoard {
        match self.stm {
            Color::White => RANK_5,
            Color::Black => RANK_4,
        }
    }

    /// Return the square on which our kingside rook starts
    pub fn ksr_start_sq(&self) -> BitBoard {
        match self.stm {
            Color::White => bb::H1,
            Color::Black => bb::H8,
        }
    }

    /// Return the square on which our queenside rook starts
    pub fn qsr_start_sq(&self) -> BitBoard {
        match self.stm {
            Color::White => bb::A1,
            Color::Black => bb::A8,
        }
    }

    /// Translate the provided bitboard in the direction of a single pawn push
    pub fn push_one(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.north_one(),
            Color::Black => bb.south_one(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn left capture
    pub fn l_cap(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.nort_west(),
            Color::Black => bb.sout_west(),
        }
    }

    /// Translate the provided bitboard in the direction of a pawn right capture
    pub fn r_cap(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.nort_east(),
            Color::Black => bb.sout_east(),
        }
    }

    /// Return a BB pushed back one square towards backrank
    pub fn back_one(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.south_one(),
            Color::Black => bb.north_one(),
        }
    }

    /// Return a BB pushed back two squares towards the backrank
    pub fn back_two(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.south_two(),
            Color::Black => bb.north_two(),
        }
    }

    /// Return a BB pushed back one square opposite the left capture direction
    pub fn l_cap_back(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.sout_east(),
            Color::Black => bb.nort_east(),
        }
    }

    /// Return the right capture pawn sources from a map of target squares
    pub fn r_cap_back(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.sout_west(),
            Color::Black => bb.nort_west(),
        }
    }

    /// Return the axis on which left capture pins occur
    pub fn l_cap_axis(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.lookup_antidiagonal_mask(),
            Color::Black => bb.lookup_diagonal_mask(),
        }
    }

    /// Return the axis on which right capture pins occur
    pub fn r_cap_axis(&self, bb: BitBoard) -> BitBoard {
        match self.stm {
            Color::White => bb.lookup_diagonal_mask(),
            Color::Black => bb.lookup_antidiagonal_mask(),
        }
    }

    /// Return the mask of the squares the king must traverse to castle kingside
    pub fn ksc_mask(&self) -> BitBoard {
        const WHITE: BitBoard = BitBoard(bb::F1.0 | bb::G1.0);
        const BLACK: BitBoard = BitBoard(bb::F8.0 | bb::G8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares the king must traverse to castle
    /// queenside so must be safe
    pub fn qsc_safety_mask(&self) -> BitBoard {
        const WHITE: BitBoard = BitBoard(bb::C1.0 | bb::D1.0);
        const BLACK: BitBoard = BitBoard(bb::C8.0 | bb::D8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }

    /// Return the mask of the squares in between the king and the rook which
    /// must be free in order to castle
    pub fn qsc_free_mask(&self) -> BitBoard {
        const WHITE: BitBoard = BitBoard(bb::B1.0 | bb::C1.0 | bb::D1.0);
        const BLACK: BitBoard = BitBoard(bb::B8.0 | bb::C8.0 | bb::D8.0);
        match self.stm {
            Color::White => WHITE,
            Color::Black => BLACK,
        }
    }
}
