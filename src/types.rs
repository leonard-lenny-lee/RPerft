use super::*;

#[derive(Clone, Copy)]
pub enum PieceType {
    All,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
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
}

pub enum MoveType {
    Quiet,
    DoublePawnPush,
    Castle(CastleType),
    Capture,
    EnPassant,
    Promotion(PieceType),
    PromotionCapture(PieceType),
}

pub enum CastleType {
    Short,
    Long,
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
