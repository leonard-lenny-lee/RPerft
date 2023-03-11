use super::*;

const FROM_TO: u8 = 63; // xx111111
const SPECIAL_1: u8 = 128; // 10xxxxxx
const SPECIAL_2: u8 = 64; // 01xxxxxx
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

pub struct MoveList {
    move_list: Vec<Move>,
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            // Based on an average branching factor of 35
            move_list: Vec::with_capacity(45),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.move_list.iter()
    }

    pub fn len(&self) -> usize {
        self.move_list.len()
    }

    pub fn add_quiet_move(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_quiet_move(target, src))
    }

    pub fn add_double_pawn_push(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_double_pawn_push(target, src))
    }

    pub fn add_short_castle(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_short_castle(target, src))
    }

    pub fn add_long_castle(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_long_castle(target, src))
    }

    pub fn add_capture(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_capture(target, src))
    }

    pub fn add_en_passant_capture(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_ep_capture(target, src))
    }

    pub fn add_promotions(&mut self, target: BB, src: BB) {
        self.move_list.push(Move::new_queen_promotion(target, src));
        self.move_list.push(Move::new_knight_promotion(target, src));
        self.move_list.push(Move::new_rook_promotion(target, src));
        self.move_list.push(Move::new_bishop_promotion(target, src))
    }

    pub fn add_promotion_captures(&mut self, target: BB, src: BB) {
        self.move_list
            .push(Move::new_queen_promo_capture(target, src));
        self.move_list
            .push(Move::new_knight_promo_capture(target, src));
        self.move_list
            .push(Move::new_rook_promo_capture(target, src));
        self.move_list
            .push(Move::new_bishop_promo_capture(target, src))
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.move_list.pop()
    }

    pub fn find(&self, mv: String) -> Option<Move> {
        for m in self.move_list.iter() {
            if mv == m.to_algebraic() {
                return Some(*m);
            }
        }
        return None;
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        self.move_list.index(index)
    }
}

#[derive(Clone, Copy)]
pub struct Move {
    word_one: u8,
    word_two: u8,
}

impl Move {
    pub fn new_null() -> Move {
        return Move {
            word_one: 0,
            word_two: 0,
        };
    }

    pub fn from_words(word_one: u8, word_two: u8) -> Move {
        return Move { word_one, word_two };
    }

    fn new_quiet_move(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src),
            word_two: Move::encode_square(target),
        };
    }

    fn new_double_pawn_push(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src),
            word_two: Move::encode_square(target) | SPECIAL_2,
        };
    }

    fn new_short_castle(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src),
            word_two: Move::encode_square(target) | SPECIAL_1,
        };
    }

    fn new_long_castle(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src),
            word_two: Move::encode_square(target) | SPECIAL_X,
        };
    }

    fn new_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_2,
            word_two: Move::encode_square(target),
        };
    }

    fn new_ep_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_2,
            word_two: Move::encode_square(target) | SPECIAL_2,
        };
    }

    fn new_knight_promotion(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_1,
            word_two: Move::encode_square(target),
        };
    }

    fn new_bishop_promotion(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_1,
            word_two: Move::encode_square(target) | SPECIAL_2,
        };
    }

    fn new_rook_promotion(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_1,
            word_two: Move::encode_square(target) | SPECIAL_1,
        };
    }

    fn new_queen_promotion(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_1,
            word_two: Move::encode_square(target) | SPECIAL_X,
        };
    }

    fn new_knight_promo_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_X,
            word_two: Move::encode_square(target),
        };
    }

    fn new_bishop_promo_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_X,
            word_two: Move::encode_square(target) | SPECIAL_2,
        };
    }

    fn new_rook_promo_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_X,
            word_two: Move::encode_square(target) | SPECIAL_1,
        };
    }

    fn new_queen_promo_capture(target: BB, src: BB) -> Move {
        return Move {
            word_one: Move::encode_square(src) | SPECIAL_X,
            word_two: Move::encode_square(target) | SPECIAL_X,
        };
    }

    fn encode_square(square: BB) -> u8 {
        square.to_index_u8()
    }

    /// Decode the target into a one bit bitmask
    pub fn target(&self) -> BB {
        BB::from_index((self.word_two & FROM_TO).into())
    }

    /// Decode the source into a one bit bitmask
    pub fn src(&self) -> BB {
        BB::from_index((self.word_one & FROM_TO).into())
    }

    pub fn is_quiet(&self) -> bool {
        self.word_one & SPECIAL_X == 0 && self.word_two & SPECIAL_X == 0
    }

    /// Decode if the piece is a capture
    pub fn is_capture(&self) -> bool {
        self.word_one & SPECIAL_2 != 0
    }

    pub fn is_promotion(&self) -> bool {
        self.word_one & SPECIAL_1 != 0
    }

    pub fn is_castle(&self) -> bool {
        self.word_two & SPECIAL_1 != 0 && self.word_one & SPECIAL_X == 0
    }

    pub fn is_short_castle(&self) -> bool {
        self.word_one & SPECIAL_X == 0 && self.word_two & SPECIAL_X == SPECIAL_1
    }

    pub fn is_long_castle(&self) -> bool {
        self.word_one & SPECIAL_X == 0 && self.word_two & SPECIAL_X == SPECIAL_X
    }

    pub fn is_en_passant(&self) -> bool {
        self.word_one & SPECIAL_X == SPECIAL_2 && self.word_two & SPECIAL_X == SPECIAL_2
    }

    pub fn is_double_pawn_push(&self) -> bool {
        self.word_one & SPECIAL_X == 0 && self.word_two & SPECIAL_X == SPECIAL_2
    }

    pub fn word_one(&self) -> u8 {
        self.word_one
    }

    pub fn word_two(&self) -> u8 {
        self.word_two
    }

    pub fn flag_one(&self) -> u8 {
        self.word_one & SPECIAL_X
    }

    pub fn flag_two(&self) -> u8 {
        self.word_two & SPECIAL_X
    }

    pub fn is_null(&self) -> bool {
        self.word_one == 0 && self.word_two == 0
    }

    pub fn promotion_piece(&self) -> usize {
        match self.word_two & SPECIAL_X {
            0 => Piece::Knight.value(),
            SPECIAL_1 => Piece::Rook.value(),
            SPECIAL_2 => Piece::Bishop.value(),
            SPECIAL_X => Piece::Queen.value(),
            _ => 0,
        }
    }

    pub fn to_algebraic(&self) -> String {
        format!(
            "{}{}{}",
            self.src().to_algebraic(),
            self.target().to_algebraic(),
            if self.is_promotion() {
                match self.promotion_piece() {
                    2 => "r",
                    3 => "n",
                    4 => "b",
                    5 => "q",
                    _ => "",
                }
            } else {
                ""
            }
        )
    }
}
