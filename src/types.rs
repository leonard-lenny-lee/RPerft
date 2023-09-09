use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Piece {
    #[default]
    Any = 0,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Rook,
    Piece::Knight,
    Piece::Bishop,
    Piece::Queen,
    Piece::King,
];

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White = 0,
    Black,
}

impl Piece {
    pub fn is_slider(&self) -> bool {
        return matches!(self, Self::Bishop | Self::Rook | Self::Queen);
    }
}

// Bitflags as discriminants
#[repr(u16)]
#[derive(Debug, Clone, Copy, Default)]
pub enum MoveType {
    #[default]
    Quiet = 0x0000,
    DoublePawnPush = 0x1000,
    ShortCastle = 0x2000,
    LongCastle = 0x3000,
    Capture = 0x4000,
    EnPassant = 0x5000,
    KnightPromotion = 0x8000,
    BishopPromotion = 0x9000,
    RookPromotion = 0xa000,
    QueenPromotion = 0xb000,
    KnightPromotionCapture = 0xc000,
    BishopPromotionCapture = 0xd000,
    RookPromotionCapture = 0xe000,
    QueenPromotionCapture = 0xf000,
}

pub const PROMOTION_MOVE_TYPES: [MoveType; 4] = [
    MoveType::KnightPromotion,
    MoveType::BishopPromotion,
    MoveType::RookPromotion,
    MoveType::QueenPromotion,
];

pub const PROMOTION_CAPTURE_MOVE_TYPES: [MoveType; 4] = [
    MoveType::KnightPromotionCapture,
    MoveType::BishopPromotionCapture,
    MoveType::RookPromotionCapture,
    MoveType::QueenPromotionCapture,
];

pub enum GeneratorType {
    Captures,           // Captures and queen promotions
    Evasions(BitBoard), // Check evasions when stm is in check
    NonEvasions,        // All captures and non captures
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
