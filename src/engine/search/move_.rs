use crate::disc;

use super::*;

const FROM_TO: u8 = 63;    // xx111111
const SPECIAL_1: u8 = 128; // 10xxxxxx
const SPECIAL_2: u8 = 64;  // 01xxxxxx
const SPECIAL_X: u8 = 192; // 11xxxxxx

/*
    Moves are encoded in two 8 bit integers.
    Bits 0-5 of word_one and word_two encode the source
    and target square, respectively. Bits 6 and 7 encode
    special move flags with the encoding below:
    CODES
    -----
    1  2  <- word
    76 76 <- index
    00 00 - quiet moves
    00 01 - double pawn push
    00 10 - king castle
    00 11 - queen castle
    x1 xx - capture flag
    01 01 - en passant capture
    1x xx - promotion flag
    1x 00 - knight promotion
    1x 01 - bishop promotion
    1x 10 - rook promotion
    1x 11 - queen promotion
*/

pub struct Move {
    word_one: u8,
    word_two: u8,
}

impl Move {

    pub fn new_quiet_move(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src),
            word_two: Move::encode_target(target),
        }
    }

    pub fn new_double_pawn_push(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src),
            word_two: Move::encode_target(target) | SPECIAL_2
        }
    }

    pub fn new_short_castle(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src),
            word_two: Move::encode_target(target) | SPECIAL_1
        }
    }

    pub fn new_long_castle(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src),
            word_two: Move::encode_target(target) | SPECIAL_X
        }
    }

    pub fn new_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_2,
            word_two: Move::encode_target(target)
        }
    }

    pub fn new_ep_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_2,
            word_two: Move::encode_target(target) | SPECIAL_2
        }
    }

    pub fn new_knight_promotion(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_1,
            word_two: Move::encode_target(target)
        }
    }

    pub fn new_bishop_promotion(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_1,
            word_two: Move::encode_target(target) | SPECIAL_2
        }
    }

    pub fn new_rook_promotion(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_1,
            word_two: Move::encode_target(target) | SPECIAL_1
        }
    }

    pub fn new_queen_promotion(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_1,
            word_two: Move::encode_target(target) | SPECIAL_X
        }
    }

    pub fn new_knight_promo_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_X,
            word_two: Move::encode_target(target)
        }
    }

    pub fn new_bishop_promo_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_X,
            word_two: Move::encode_target(target) | SPECIAL_2
        }
    }

    pub fn new_rook_promo_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_X,
            word_two: Move::encode_target(target) | SPECIAL_1
        }
    }

    pub fn new_queen_promo_capture(target: u64, src: u64) -> Move {
        return Move {
            word_one: Move::encode_src(src) | SPECIAL_X,
            word_two: Move::encode_target(target) | SPECIAL_X
        }
    }

    fn encode_target(target: u64) -> u8 {
        bt::ilsb_u8(target)
    }

    fn encode_src(src: u64) -> u8 {
        bt::ilsb_u8(src)
    } 

    /// Decode the target into a one bit bitmask
    pub fn target(&self) -> u64 {
        1 << (self.word_two & FROM_TO)
    }

    /// Decode the source into a one bit bitmask
    pub fn src(&self) -> u64 {
        1 << (self.word_one & FROM_TO)
    }

    pub fn is_quiet(&self) -> bool {
        self.word_one & SPECIAL_X == 0
        && self.word_two & SPECIAL_X == 0
    }

    /// Decode if the piece is a capture
    pub fn is_capture(&self) -> bool {
        self.word_one & SPECIAL_2 != 0
    }

    pub fn is_promotion(&self) -> bool {
        self.word_one & SPECIAL_1 != 0
    }

    pub fn is_castle(&self) -> bool {
        self.word_two & SPECIAL_1 != 0 
        && self.word_one & SPECIAL_X == 0
    }

    pub fn is_short_castle(&self) -> bool {
        self.word_one & SPECIAL_X == 0
        && self.word_two & SPECIAL_X == SPECIAL_1
    }

    pub fn is_long_castle(&self) -> bool {
        self.word_one & SPECIAL_X == 0
        && self.word_two & SPECIAL_X == SPECIAL_X
    }

    pub fn is_en_passant(&self) -> bool {
        self.word_one & SPECIAL_X == SPECIAL_2
        && self.word_two & SPECIAL_X == SPECIAL_2
    }

    pub fn is_double_pawn_push(&self) -> bool {
        self.word_one & SPECIAL_X == 0
        && self.word_two & SPECIAL_X == SPECIAL_2
    }

    pub fn promotion_piece(&self) -> usize {
        match self.word_two & SPECIAL_X {
            0 => disc!(Piece::Knight),
            SPECIAL_1 => disc!(Piece::Rook),
            SPECIAL_2 => disc!(Piece::Bishop),
            SPECIAL_X => disc!(Piece::Queen),
            _ => 0
        }
    }

}