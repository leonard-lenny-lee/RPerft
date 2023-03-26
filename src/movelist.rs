use super::*;
use types::{CastleType, MoveType, PieceType};

pub trait MoveList {
    fn new() -> Self;

    fn iter(&self) -> std::slice::Iter<Move>;

    fn len(&self) -> usize;

    fn add_quiet(&mut self, from: BB, to: BB);

    fn add_double_pawn_push(&mut self, from: BB, to: BB);

    fn add_short_castle(&mut self, from: BB, to: BB);

    fn add_long_castle(&mut self, from: BB, to: BB);

    fn add_capture(&mut self, from: BB, to: BB);

    fn add_ep(&mut self, from: BB, to: BB);

    fn add_promotions(&mut self, from: BB, to: BB);

    fn add_promo_captures(&mut self, from: BB, to: BB);
}

/// Scores moves as they are added and orders them to optimize pruning
pub struct OrderedList {
    _movelist: Vec<Move>,
}

impl MoveList for OrderedList {
    fn new() -> Self {
        todo!()
    }

    fn iter(&self) -> std::slice::Iter<Move> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn add_quiet(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_double_pawn_push(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_short_castle(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_long_castle(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_capture(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_ep(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_promotions(&mut self, _from: BB, _to: BB) {
        todo!()
    }

    fn add_promo_captures(&mut self, _from: BB, _to: BB) {
        todo!()
    }
}

pub struct UnorderedList {
    movelist: Vec<Move>,
}

impl MoveList for UnorderedList {
    fn new() -> Self {
        Self {
            // Based on an average branching factor of 35
            movelist: Vec::with_capacity(45),
        }
    }

    fn iter(&self) -> std::slice::Iter<Move> {
        self.movelist.iter()
    }

    fn len(&self) -> usize {
        self.movelist.len()
    }

    fn add_quiet(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_quiet(from, to))
    }

    fn add_double_pawn_push(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_double_pawn_push(from, to))
    }

    fn add_short_castle(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_short_castle(from, to))
    }

    fn add_long_castle(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_long_castle(from, to))
    }

    fn add_capture(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_capture(from, to))
    }

    fn add_ep(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_ep_capture(from, to))
    }

    fn add_promotions(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_queen_promotion(from, to));
        self.movelist.push(Move::new_knight_promotion(from, to));
        self.movelist.push(Move::new_rook_promotion(from, to));
        self.movelist.push(Move::new_bishop_promotion(from, to));
    }

    fn add_promo_captures(&mut self, from: BB, to: BB) {
        self.movelist.push(Move::new_queen_promo_capture(from, to));
        self.movelist.push(Move::new_knight_promo_capture(from, to));
        self.movelist.push(Move::new_rook_promo_capture(from, to));
        self.movelist.push(Move::new_bishop_promo_capture(from, to));
    }
}

impl UnorderedList {
    pub fn pop(&mut self) -> Option<Move> {
        self.movelist.pop()
    }

    pub fn find(&self, mv: String) -> Option<Move> {
        for m in self.movelist.iter() {
            if mv == m.to_algebraic() {
                return Some(*m);
            }
        }
        return None;
    }
}

impl std::ops::Index<usize> for UnorderedList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        self.movelist.index(index)
    }
}

/*
    Moves are encoded in an 16 bit integer.
    Bits 0-5 and 6-11 encode the source and target square, respectively.
    Bits 12-15 encode special move flags with the encoding below:

    FLAGS
    -----
    0000 - quiet moves
    0001 - double pawn push
    0010 - king castle
    0011 - queen castle
    0100 - captures
    0101 - en passant capture
    0110 - NONE
    0111 - NONE
    1000 - knight promotion
    1001 - bishop promotion
    1010 - rook promotion
    1011 - queen promotion
    1100 - knight-promo capture
    1101 - bishop-promo capture
    1110 - rook-promo capture
    1111 - queen-promo capture

    x1xx - capture flag
    1xxx - promotion flag
*/

// Special move flags
const QUIET: u16 = 0x0000;
const DOUBLE_PAWN_PUSH: u16 = 0x1000;
const SHORT_CASTLE: u16 = 0x2000;
const LONG_CASTLE: u16 = 0x3000;
const CAPTURE: u16 = 0x4000;
const ENPASSANT: u16 = 0x5000;
const KNIGHT_PROMO: u16 = 0x8000;
const BISHOP_PROMO: u16 = 0x9000;
const ROOK_PROMO: u16 = 0xa000;
const QUEEN_PROMO: u16 = 0xb000;
const KNIGHT_PROMO_CAPTURE: u16 = 0xc000;
const BISHOP_PROMO_CAPTURE: u16 = 0xd000;
const ROOK_PROMO_CAPTURE: u16 = 0xe000;
const QUEEN_PROMO_CAPTURE: u16 = 0xf000;

const CAPTURE_FLAG: u16 = 0x4000;
const PROMO_FLAG: u16 = 0x8000;

const SRC: u16 = 0x003f;
const TARGET: u16 = 0x0fc0;
const FLAGS: u16 = 0xf000;

#[derive(Clone, Copy)]
pub struct Move(pub u16);

impl Move {
    pub fn new_null() -> Move {
        return Move(0);
    }

    pub fn from_uint16(word: u16) -> Move {
        return Move(word);
    }

    fn new_quiet(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to));
    }

    fn new_double_pawn_push(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | DOUBLE_PAWN_PUSH);
    }

    fn new_short_castle(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | SHORT_CASTLE);
    }

    fn new_long_castle(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | LONG_CASTLE);
    }

    fn new_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | CAPTURE);
    }

    fn new_ep_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ENPASSANT);
    }

    fn new_knight_promotion(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | KNIGHT_PROMO);
    }

    fn new_bishop_promotion(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | BISHOP_PROMO);
    }

    fn new_rook_promotion(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ROOK_PROMO);
    }

    fn new_queen_promotion(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | QUEEN_PROMO);
    }

    fn new_knight_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | KNIGHT_PROMO_CAPTURE);
    }

    fn new_bishop_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | BISHOP_PROMO_CAPTURE);
    }

    fn new_rook_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ROOK_PROMO_CAPTURE);
    }

    fn new_queen_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | QUEEN_PROMO_CAPTURE);
    }

    fn encode_move(from: BB, to: BB) -> u16 {
        return from.to_uint16_sq() | (to.to_uint16_sq() << 6);
    }

    /// Decode the target into a one bit bitmask
    pub fn to(&self) -> BB {
        BB::from_sq(((self.0 & TARGET) >> 6).into())
    }

    /// Decode the source into a one bit bitmask
    pub fn from(&self) -> BB {
        BB::from_sq((self.0 & SRC).into())
    }

    /// Decode the type of move
    pub fn movetype(&self) -> MoveType {
        use CastleType::*;
        use MoveType::*;
        use PieceType::*;

        match self.0 & FLAGS {
            QUIET => Quiet,
            DOUBLE_PAWN_PUSH => DoublePawnPush,
            SHORT_CASTLE => Castle(Short),
            LONG_CASTLE => Castle(Long),
            CAPTURE => Capture,
            ENPASSANT => EnPassant,
            KNIGHT_PROMO => Promotion(Knight),
            BISHOP_PROMO => Promotion(Bishop),
            ROOK_PROMO => Promotion(Rook),
            QUEEN_PROMO => Promotion(Queen),
            KNIGHT_PROMO_CAPTURE => PromotionCapture(Knight),
            BISHOP_PROMO_CAPTURE => PromotionCapture(Bishop),
            ROOK_PROMO_CAPTURE => PromotionCapture(Rook),
            QUEEN_PROMO_CAPTURE => PromotionCapture(Queen),
            _ => panic!("Unrecognised move type encoding"),
        }
    }

    /// Decode if the move encodes a capture of any sort
    pub fn is_capture(&self) -> bool {
        return self.0 & CAPTURE_FLAG != 0;
    }

    /// Decode if the move encodes a promotion of any sort
    pub fn is_promotion(&self) -> bool {
        return self.0 & PROMO_FLAG != 0;
    }

    /// Is the move a null move
    pub fn is_null(&self) -> bool {
        return self.0 == 0;
    }

    /// What kind of promotion is encoded
    pub fn promotion_piece(&self) -> Option<PieceType> {
        match self.0 & FLAGS {
            KNIGHT_PROMO | KNIGHT_PROMO_CAPTURE => Some(PieceType::Knight),
            ROOK_PROMO | ROOK_PROMO_CAPTURE => Some(PieceType::Rook),
            BISHOP_PROMO | BISHOP_PROMO_CAPTURE => Some(PieceType::Bishop),
            QUEEN_PROMO | QUEEN_PROMO_CAPTURE => Some(PieceType::Queen),
            _ => None,
        }
    }

    pub fn to_algebraic(&self) -> String {
        format!(
            "{}{}{}",
            self.from().to_algebraic(),
            self.to().to_algebraic(),
            if self.is_promotion() {
                if let Some(p) = self.promotion_piece() {
                    match p {
                        PieceType::Rook => "r",
                        PieceType::Knight => "n",
                        PieceType::Bishop => "b",
                        PieceType::Queen => "q",
                        _ => "",
                    }
                } else {
                    ""
                }
            } else {
                ""
            }
        )
    }
}
