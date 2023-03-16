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
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
