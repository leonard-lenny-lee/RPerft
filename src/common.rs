/// This file contains hardcoded masks and Enums which are required by many
/// other parts of the program.

use strum_macros::EnumIter;

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

pub const FILLED_BB: u64 = 0xffffffffffffffff;
pub const EMPTY_BB: u64 = 0x0;
pub const ROOK_START: u64 = (FILE_A | FILE_H) & (RANK_1 | RANK_8);
pub const WQROOK: u64 = 1;
pub const WKROOK: u64 = 1 << 7;
pub const BQROOK: u64 = 1 << 56;
pub const BKROOK: u64 = 1 << 63;

// Castle masks [KingsideMask, KingsideTarget, QueensideMask, QueensideTarget]
pub const W_CASTLE: [u64; 4] = [0x60, 0x40, 0xe, 0x4];
pub const B_CASTLE: [u64; 4] = [0x6000000000000000, 0x4000000000000000, 0xe00000000000000, 0x400000000000000];


pub enum ASCIIBases {
    LowerA = 97,
    UpperA = 65,
    Zero = 48,
}

pub enum Promotion {
    None = 0,
    Rook = 2,
    Knight = 3,
    Bishop = 4,
    Queen = 5,
}

impl Promotion {
    pub fn iterator() -> Vec<Promotion> {
        use Promotion::*;
        return vec![Rook, Knight, Bishop, Queen];
    }
}

pub enum SpecialMove {
    None,
    Promotion,
    EnPassant,
    Castling,
    DoublePush,
}

#[derive(EnumIter)]
pub enum PawnMove {
    SinglePush,
    DoublePush,
    CaptureLeft,
    CaptureRight,
}

#[derive(EnumIter)]
pub enum JumpingPiece {
    Knight = 3,
    King = 6,
}

#[derive(Clone, Copy)]
pub enum Piece {
    Any,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

impl Piece {
    pub fn iter_pieces() -> Vec<Piece> {
        use Piece::*;
        return vec![Pawn, Rook, Knight, Bishop, Queen, King];
    }
}

#[derive(EnumIter)]
pub enum SlidingPiece {
    Rook = 2,
    Bishop = 4,
    Queen = 5,
}

pub enum Color {
    White, Black
}

#[macro_export]
/// Returns the discriminant of an enum for indexing -> usize
macro_rules! disc {
    ($enum:expr) => {
        $enum as usize
    };
}