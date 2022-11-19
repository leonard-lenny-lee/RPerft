/// This file contains hardcoded masks and Enums which are required by many
/// other parts of the program.

pub mod bittools;

pub const DEFAULT_FEN: &str= "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Rank Masks
pub const RANK_1: u64 = 0x00000000000000ff;
pub const RANK_2: u64 = 0x000000000000ff00;
pub const RANK_3: u64 = 0x0000000000ff0000;
pub const RANK_4: u64 = 0x00000000ff000000;
pub const RANK_5: u64 = 0x000000ff00000000;
pub const RANK_6: u64 = 0x0000ff0000000000;
pub const RANK_7: u64 = 0x00ff000000000000;
pub const RANK_8: u64 = 0xff00000000000000;

// File Masks
pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

// Castle masks [KingsideMask, KingsideTarget, QueensideMask, QueensideTarget]
pub const W_CASTLE: [u64; 4] = [0x60, 0x40, 0xe, 0x4];
pub const B_CASTLE: [u64; 4] = [0x6000000000000000, 0x4000000000000000, 0xe00000000000000, 0x400000000000000];


pub enum ASCIIBases {
    LowerA = 97, UpperA = 65, Zero = 48,
}

pub enum PromotionPiece {
    None, Rook, Knight, Bishop, Queen,
}

impl PromotionPiece {
    pub fn iterator() -> Vec<PromotionPiece> {
        use PromotionPiece::*;
        return vec![Rook, Knight, Bishop, Queen];
    }
}

pub enum SpecialMove {
    None, Promotion, EnPassant, Castling,
}

pub enum PawnMove {
    SinglePush, DoublePush, CaptureLeft, CaptureRight,
}

pub enum JumpingPiece {
    Knight = 3, King = 6,
}

impl JumpingPiece {
    pub fn iterator() -> Vec<JumpingPiece> {
        use JumpingPiece::*;
        return vec![Knight, King];
    }
}

pub enum SlidingPiece {
    Rook = 2, Bishop = 4, Queen = 5,
}

impl SlidingPiece {
    pub fn iterator() -> Vec<SlidingPiece> {
        use SlidingPiece::*;
        return vec![Bishop, Rook, Queen]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Piece {
    Any, Pawn, Rook, Knight, Bishop, Queen, King
}

impl Piece {

    pub fn iter_pieces() -> Vec<Piece> {
        use Piece::*;
        return vec![Pawn, Rook, Knight, Bishop, Queen, King];
    }
}

pub enum Color {
    White, Black
}