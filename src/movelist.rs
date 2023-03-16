use super::*;
use types::PieceType;

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

pub enum MoveType {
    Quiet,
    DoublePawnPush,
    ShortCastle,
    LongCastle,
    Capture,
    EnPassant,
    KnightPromo,
    BishopPromo,
    RookPromo,
    QueenPromo,
    KnightPromoCapture,
    BishopPromoCapture,
    RookPromoCapture,
    QueenPromoCapture,
}

#[derive(Clone, Copy)]
pub struct Move(pub u16);

impl Move {
    pub fn new_null() -> Move {
        return Move(0);
    }

    pub fn from_uint16(word: u16) -> Move {
        return Move(word);
    }

    fn new_quiet_move(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target));
    }

    fn new_double_pawn_push(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | DOUBLE_PAWN_PUSH);
    }

    fn new_short_castle(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | SHORT_CASTLE);
    }

    fn new_long_castle(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | LONG_CASTLE);
    }

    fn new_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | CAPTURE);
    }

    fn new_ep_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | ENPASSANT);
    }

    fn new_knight_promotion(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | KNIGHT_PROMO);
    }

    fn new_bishop_promotion(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | BISHOP_PROMO);
    }

    fn new_rook_promotion(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | ROOK_PROMO);
    }

    fn new_queen_promotion(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | QUEEN_PROMO);
    }

    fn new_knight_promo_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | KNIGHT_PROMO_CAPTURE);
    }

    fn new_bishop_promo_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | BISHOP_PROMO_CAPTURE);
    }

    fn new_rook_promo_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | ROOK_PROMO_CAPTURE);
    }

    fn new_queen_promo_capture(target: BB, src: BB) -> Move {
        return Move(Move::encode_move(src, target) | QUEEN_PROMO_CAPTURE);
    }

    fn encode_move(src: BB, target: BB) -> u16 {
        return src.to_index_uint16() | (target.to_index_uint16() << 6);
    }

    /// Decode the target into a one bit bitmask
    pub fn target(&self) -> BB {
        BB::from_index(((self.0 & TARGET) >> 6).into())
    }

    /// Decode the source into a one bit bitmask
    pub fn src(&self) -> BB {
        BB::from_index((self.0 & SRC).into())
    }

    /// Decode the type of move
    pub fn movetype(&self) -> MoveType {
        match self.0 & FLAGS {
            QUIET => MoveType::Quiet,
            DOUBLE_PAWN_PUSH => MoveType::DoublePawnPush,
            SHORT_CASTLE => MoveType::ShortCastle,
            LONG_CASTLE => MoveType::LongCastle,
            CAPTURE => MoveType::Capture,
            ENPASSANT => MoveType::EnPassant,
            KNIGHT_PROMO => MoveType::KnightPromo,
            BISHOP_PROMO => MoveType::BishopPromo,
            ROOK_PROMO => MoveType::RookPromo,
            QUEEN_PROMO => MoveType::QueenPromo,
            KNIGHT_PROMO_CAPTURE => MoveType::KnightPromoCapture,
            BISHOP_PROMO_CAPTURE => MoveType::BishopPromoCapture,
            ROOK_PROMO_CAPTURE => MoveType::RookPromoCapture,
            QUEEN_PROMO_CAPTURE => MoveType::QueenPromoCapture,
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
            self.src().to_algebraic(),
            self.target().to_algebraic(),
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
