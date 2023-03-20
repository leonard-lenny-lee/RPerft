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

    pub fn is_slider(&self) -> bool {
        return matches!(self, Self::Bishop | Self::Rook | Self::Queen);
    }
}

pub enum MoveType {
    Quiet,
    DoublePawnPush,
    Castle { is_long: bool },
    Capture,
    EnPassant,
    Promotion(PieceType),
    PromotionCapture(PieceType),
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
