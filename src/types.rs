use super::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum PieceType {
    #[default]
    All,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl PieceType {
    /// Returns an iterator that iterates through the pieces only
    pub fn iterpieces() -> std::slice::Iter<'static, PieceType> {
        use PieceType::*;
        static PIECES: [PieceType; 6] = [Pawn, Rook, Knight, Bishop, Queen, King];
        return PIECES.iter();
    }

    pub fn promopieces() -> std::slice::Iter<'static, Self> {
        use PieceType::*;
        static PIECES: [PieceType; 4] = [Queen, Knight, Rook, Bishop];
        return PIECES.iter();
    }

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
    NPromotion = 0x8000,
    BPromotion = 0x9000,
    RPromotion = 0xa000,
    QPromotion = 0xb000,
    NPromoCapture = 0xc000,
    BPromoCapture = 0xd000,
    RPromoCapture = 0xe000,
    QPromoCapture = 0xf000,
}

impl MoveType {
    pub fn iter_promos() -> std::slice::Iter<'static, Self> {
        use MoveType::*;
        static TYPES: [MoveType; 4] = [NPromotion, BPromotion, RPromotion, QPromotion];
        return TYPES.iter();
    }

    pub fn iter_promo_captures() -> std::slice::Iter<'static, Self> {
        use MoveType::*;
        static TYPES: [MoveType; 4] = [NPromoCapture, BPromoCapture, RPromoCapture, QPromoCapture];
        return TYPES.iter();
    }
}

pub enum GenType {
    Captures,     // Captures and queen promotions
    Evasions(BB), // Check evasions when stm is in check
    NonEvasions,  // All captures and non captures
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}

pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    PV,  // Score is Exact
    Cut, // Score is Lower Bound
    All, // Score is Upper Bound
}
