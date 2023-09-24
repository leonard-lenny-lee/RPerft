/// Implementation of a White / Black state machine to execute turn dependent logic
use super::*;
use constants::bb;
use constants::rank::*;

const W_KSC_MASK: BitBoard = BitBoard(bb::F1.0 | bb::G1.0);
const B_KSC_MASK: BitBoard = BitBoard(bb::F8.0 | bb::G8.0);
const W_QSC_SAFETY_MASK: BitBoard = BitBoard(bb::C1.0 | bb::D1.0);
const B_QSC_SAFETY_MASK: BitBoard = BitBoard(bb::C8.0 | bb::D8.0);
const W_QSC_FREE_MASK: BitBoard = BitBoard(bb::B1.0 | bb::C1.0 | bb::D1.0);
const B_QSC_FREE_MASK: BitBoard = BitBoard(bb::B8.0 | bb::C8.0 | bb::D8.0);

pub trait Color {
    fn rank_7() -> BitBoard;
    fn rank_2() -> BitBoard;
    fn rank_3() -> BitBoard;
    fn rank_1() -> BitBoard;
    fn rank_5() -> BitBoard;
    fn ksr_start_sq() -> BitBoard;
    fn qsr_start_sq() -> BitBoard;
    fn push_one(bb: BitBoard) -> BitBoard;
    fn l_cap(bb: BitBoard) -> BitBoard;
    fn r_cap(bb: BitBoard) -> BitBoard;
    fn back_one(bb: BitBoard) -> BitBoard;
    fn back_two(bb: BitBoard) -> BitBoard;
    fn l_cap_back(bb: BitBoard) -> BitBoard;
    fn r_cap_back(bb: BitBoard) -> BitBoard;
    fn l_cap_axis(bb: BitBoard) -> BitBoard;
    fn r_cap_axis(bb: BitBoard) -> BitBoard;
    fn ksc_mask() -> BitBoard;
    fn qsc_safety_mask() -> BitBoard;
    fn qsc_free_mask() -> BitBoard;
    fn cap_back(bb: BitBoard) -> BitBoard;
}

pub struct White;
pub struct Black;

impl Color for White {
    fn rank_7() -> BitBoard {
        RANK_7
    }

    fn rank_2() -> BitBoard {
        RANK_2
    }

    fn rank_3() -> BitBoard {
        RANK_3
    }

    fn rank_1() -> BitBoard {
        RANK_1
    }

    fn rank_5() -> BitBoard {
        RANK_5
    }

    fn ksr_start_sq() -> BitBoard {
        bb::H1
    }

    fn qsr_start_sq() -> BitBoard {
        bb::A1
    }

    fn push_one(bb: BitBoard) -> BitBoard {
        bb.north_one()
    }

    fn l_cap(bb: BitBoard) -> BitBoard {
        bb.nort_west()
    }

    fn r_cap(bb: BitBoard) -> BitBoard {
        bb.nort_east()
    }

    fn back_one(bb: BitBoard) -> BitBoard {
        bb.south_one()
    }

    fn back_two(bb: BitBoard) -> BitBoard {
        bb.south_two()
    }

    fn l_cap_back(bb: BitBoard) -> BitBoard {
        bb.sout_east()
    }

    fn r_cap_back(bb: BitBoard) -> BitBoard {
        bb.sout_west()
    }

    fn l_cap_axis(bb: BitBoard) -> BitBoard {
        bb.lookup_antidiagonal_mask()
    }

    fn r_cap_axis(bb: BitBoard) -> BitBoard {
        bb.lookup_diagonal_mask()
    }

    fn ksc_mask() -> BitBoard {
        W_KSC_MASK
    }

    fn qsc_safety_mask() -> BitBoard {
        W_QSC_SAFETY_MASK
    }

    fn qsc_free_mask() -> BitBoard {
        W_QSC_FREE_MASK
    }

    fn cap_back(bb: BitBoard) -> BitBoard {
        bb.sout_west() | bb.sout_east()
    }
}

impl Color for Black {
    fn rank_7() -> BitBoard {
        RANK_2
    }

    fn rank_2() -> BitBoard {
        RANK_7
    }

    fn rank_3() -> BitBoard {
        RANK_6
    }

    fn rank_1() -> BitBoard {
        RANK_8
    }

    fn rank_5() -> BitBoard {
        RANK_4
    }

    fn ksr_start_sq() -> BitBoard {
        bb::H8
    }

    fn qsr_start_sq() -> BitBoard {
        bb::A8
    }

    fn push_one(bb: BitBoard) -> BitBoard {
        bb.south_one()
    }

    fn l_cap(bb: BitBoard) -> BitBoard {
        bb.sout_west()
    }

    fn r_cap(bb: BitBoard) -> BitBoard {
        bb.sout_east()
    }

    fn back_one(bb: BitBoard) -> BitBoard {
        bb.north_one()
    }

    fn back_two(bb: BitBoard) -> BitBoard {
        bb.north_two()
    }

    fn l_cap_back(bb: BitBoard) -> BitBoard {
        bb.nort_east()
    }

    fn r_cap_back(bb: BitBoard) -> BitBoard {
        bb.nort_west()
    }

    fn l_cap_axis(bb: BitBoard) -> BitBoard {
        bb.lookup_diagonal_mask()
    }

    fn r_cap_axis(bb: BitBoard) -> BitBoard {
        bb.lookup_antidiagonal_mask()
    }

    fn ksc_mask() -> BitBoard {
        B_KSC_MASK
    }

    fn qsc_safety_mask() -> BitBoard {
        B_QSC_SAFETY_MASK
    }

    fn qsc_free_mask() -> BitBoard {
        B_QSC_FREE_MASK
    }

    fn cap_back(bb: BitBoard) -> BitBoard {
        bb.nort_west() | bb.nort_east()
    }
}

impl Position {
    /// Reverse the side to move
    pub fn change_state(&mut self) {
        unsafe { self.stm = std::mem::transmute::<u8, ColorT>((self.stm as u8) ^ 1) }
        self.wtm = !self.wtm;
        std::mem::swap(&mut self.us, &mut self.them)
    }

    pub fn white_black(&self) -> (&BitBoardSet, &BitBoardSet) {
        match self.stm {
            ColorT::White => (&self.us, &self.them),
            ColorT::Black => (&self.them, &self.us),
        }
    }
}
