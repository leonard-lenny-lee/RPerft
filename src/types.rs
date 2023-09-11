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

// Bitflags as discriminants
#[repr(u16)]
#[derive(Debug, Clone, Copy, Default)]
pub enum MoveT {
    #[default]
    Quiet = 0x0000,
    DoublePawnPush = 0x1000,
    KSCastle = 0x2000,
    QSCastle = 0x3000,
    Capture = 0x4000,
    EnPassant = 0x5000,
    NPromo = 0x8000,
    BPromo = 0x9000,
    RPromo = 0xa000,
    QPromo = 0xb000,
    NPromoCapture = 0xc000,
    BPromoCapture = 0xd000,
    RPromoCapture = 0xe000,
    QPromoCapture = 0xf000,
}

pub const PROMOTION_MOVE_TYPES: [MoveT; 4] =
    [MoveT::NPromo, MoveT::BPromo, MoveT::RPromo, MoveT::QPromo];

pub const PROMOTION_CAPTURE_MOVE_TYPES: [MoveT; 4] = [
    MoveT::NPromoCapture,
    MoveT::BPromoCapture,
    MoveT::RPromoCapture,
    MoveT::QPromoCapture,
];
