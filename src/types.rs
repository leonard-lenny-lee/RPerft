use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PieceType {
    #[default]
    Any = 0,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub const PIECES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Rook,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Queen,
    PieceType::King,
];

#[derive(Clone, Copy)]
pub enum Color {
    White = 0,
    Black,
}

impl PieceType {
    pub fn is_slider(&self) -> bool {
        return matches!(self, Self::Bishop | Self::Rook | Self::Queen);
    }

    pub fn to_uci(&self) -> v_uci::UciPiece {
        use v_uci::UciPiece;

        match self {
            PieceType::Pawn => UciPiece::Pawn,
            PieceType::Rook => UciPiece::Rook,
            PieceType::Knight => UciPiece::Knight,
            PieceType::Bishop => UciPiece::Bishop,
            PieceType::Queen => UciPiece::Queen,
            PieceType::King => UciPiece::King,
            _ => panic!("invalid pt for to_uci"),
        }
    }

    // Convert to NNUE piece code
    pub fn to_nnue_pc(&self) -> usize {
        use nnue::Pieces::*;
        const MAP: [nnue::Pieces; 7] = [Blank, WPawn, WRook, WKnight, WBishop, WQueen, WKing];
        return MAP[*self as usize] as usize;
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

#[derive(Clone, Copy)]
pub enum NodeType {
    PV,  // Score is Exact
    Cut, // Score is Lower Bound
    All, // Score is Upper Bound
}
