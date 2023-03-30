use super::*;
use position::Position;
use types::{CastleType, MoveType, PieceType};

pub trait MoveList {
    fn new() -> Self;

    // fn iter(&self) -> std::slice::Iter<Move>;

    fn len(&self) -> usize;

    fn add_quiet(&mut self, from: BB, to: BB);

    fn add_double_pawn_push(&mut self, from: BB, to: BB);

    fn add_short_castle(&mut self, from: BB, to: BB);

    fn add_long_castle(&mut self, from: BB, to: BB);

    fn add_capture(&mut self, from: BB, to: BB, pos: &Position);

    fn add_ep(&mut self, from: BB, to: BB);

    fn add_promotions(&mut self, from: BB, to: BB);

    fn add_promo_captures(&mut self, from: BB, to: BB, pos: &Position);
}

// "Move valuable victim, least valuable aggressor" is a simple move ordering
// heuristic to play winning captures first. The is the lookup table
const MVV_LVA: [[i16; 7]; 7] = {
    // Based on the increasing value of pieces: P->N->B->R->Q->K
    const VALS: [i16; 7] = [0, 100, 400, 200, 300, 500, 600];

    let mut mvv_lva = [[0; 7]; 7];
    let mut victim = 0;
    while victim < 7 {
        let mut attacker = 0;
        while attacker < 7 {
            mvv_lva[victim][attacker] = VALS[victim] - VALS[attacker] / 10;
            attacker += 1;
        }
        victim += 1;
    }
    mvv_lva
};

/// Scores moves as they are added and orders them to optimize pruning
pub struct OrderedList(pub Vec<(Move, i16)>);

impl MoveList for OrderedList {
    fn new() -> Self {
        Self(Vec::with_capacity(45))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn add_quiet(&mut self, from: BB, to: BB) {
        self.0.push((Move::quiet(from, to), 0))
    }

    fn add_double_pawn_push(&mut self, from: BB, to: BB) {
        self.0.push((Move::dbl_pawn_push(from, to), 0))
    }

    fn add_short_castle(&mut self, from: BB, to: BB) {
        self.0.push((Move::short_castle(from, to), 0))
    }

    fn add_long_castle(&mut self, from: BB, to: BB) {
        self.0.push((Move::long_castle(from, to), 0))
    }

    fn add_capture(&mut self, from: BB, to: BB, pos: &Position) {
        // Score based on winning captures
        let v = pos.them.pt_at(to).unwrap() as usize;
        let a = pos.us.pt_at(from).unwrap() as usize;

        self.0.push((Move::capture(from, to), MVV_LVA[v][a]));
    }

    fn add_ep(&mut self, from: BB, to: BB) {
        const EP_SCORE: i16 = MVV_LVA[1][1];
        // Based on pawn victim and attacker values of 100 and 10
        self.0.push((Move::ep_capture(from, to), EP_SCORE))
    }

    fn add_promotions(&mut self, from: BB, to: BB) {
        // Score based on loss of a pawn and gain of promotion piece
        self.0.push((Move::q_promo(from, to), 0));
        self.0.push((Move::n_promo(from, to), 0));
        self.0.push((Move::r_promo(from, to), 0));
        self.0.push((Move::b_promo(from, to), 0));
    }

    fn add_promo_captures(&mut self, from: BB, to: BB, pos: &Position) {
        let v = pos.them.pt_at(to).unwrap() as usize;
        let score = MVV_LVA[v][1];

        self.0.push((Move::q_promo_capture(from, to), score));
        self.0.push((Move::n_promo_capture(from, to), score));
        self.0.push((Move::r_promo_capture(from, to), score));
        self.0.push((Move::b_promo_capture(from, to), score));
    }
}

impl OrderedList {
    /// Increase the score of hash and killer moves
    pub fn score(&mut self, tt_move: Move, killers: [Move; 2]) {
        // TODO Add History Heuristic
        for (mv, score) in self.0.iter_mut() {
            if tt_move.0 == mv.0 {
                *score += 1000; // Order hash moves first
            } else if killers[0].0 == mv.0 {
                *score += 50; // Primary killer
            } else if killers[1].0 == mv.0 {
                *score += 45 // Secondary killer
            }
        }
    }

    pub fn sort(&mut self) {
        self.0.sort_by_key(|mv| std::cmp::Reverse(mv.1));
    }
}

impl std::ops::Index<usize> for OrderedList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0.index(index).0
    }
}

pub struct UnorderedList(Vec<Move>);

impl MoveList for UnorderedList {
    fn new() -> Self {
        Self(Vec::with_capacity(45))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn add_quiet(&mut self, from: BB, to: BB) {
        self.0.push(Move::quiet(from, to))
    }

    fn add_double_pawn_push(&mut self, from: BB, to: BB) {
        self.0.push(Move::dbl_pawn_push(from, to))
    }

    fn add_short_castle(&mut self, from: BB, to: BB) {
        self.0.push(Move::short_castle(from, to))
    }

    fn add_long_castle(&mut self, from: BB, to: BB) {
        self.0.push(Move::long_castle(from, to))
    }

    fn add_capture(&mut self, from: BB, to: BB, _pos: &Position) {
        self.0.push(Move::capture(from, to))
    }

    fn add_ep(&mut self, from: BB, to: BB) {
        self.0.push(Move::ep_capture(from, to))
    }

    fn add_promotions(&mut self, from: BB, to: BB) {
        self.0.push(Move::q_promo(from, to));
        self.0.push(Move::n_promo(from, to));
        self.0.push(Move::r_promo(from, to));
        self.0.push(Move::b_promo(from, to));
    }

    fn add_promo_captures(&mut self, from: BB, to: BB, _pos: &Position) {
        self.0.push(Move::q_promo_capture(from, to));
        self.0.push(Move::n_promo_capture(from, to));
        self.0.push(Move::r_promo_capture(from, to));
        self.0.push(Move::b_promo_capture(from, to));
    }
}

impl UnorderedList {
    pub fn new() -> Self {
        // Based on an average branching factor of 35
        Self(Vec::with_capacity(45))
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.0.iter()
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.0.pop()
    }

    pub fn find(&self, mv: String) -> Option<Move> {
        for m in self.0.iter() {
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
        self.0.index(index)
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
    pub fn null() -> Move {
        return Move(0);
    }

    pub fn from_uint16(word: u16) -> Move {
        return Move(word);
    }

    fn quiet(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to));
    }

    fn dbl_pawn_push(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | DOUBLE_PAWN_PUSH);
    }

    fn short_castle(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | SHORT_CASTLE);
    }

    fn long_castle(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | LONG_CASTLE);
    }

    fn capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | CAPTURE);
    }

    fn ep_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ENPASSANT);
    }

    fn n_promo(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | KNIGHT_PROMO);
    }

    fn b_promo(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | BISHOP_PROMO);
    }

    fn r_promo(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ROOK_PROMO);
    }

    fn q_promo(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | QUEEN_PROMO);
    }

    fn n_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | KNIGHT_PROMO_CAPTURE);
    }

    fn b_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | BISHOP_PROMO_CAPTURE);
    }

    fn r_promo_capture(from: BB, to: BB) -> Move {
        return Move(Move::encode_move(from, to) | ROOK_PROMO_CAPTURE);
    }

    fn q_promo_capture(from: BB, to: BB) -> Move {
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
