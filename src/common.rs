/// This file contains hardcoded masks and Enums which are required by many
/// other parts of the program.
use super::*;

// Standard chess positions useful for testing
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const POSITION_2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
pub const POSITION_4: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
pub const POSITION_6: &str =
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ";

// Rank Masks
pub const RANK_1: BB = BB(0x00000000000000ff);
pub const RANK_2: BB = BB(0x000000000000ff00);
pub const RANK_3: BB = BB(0x0000000000ff0000);
pub const RANK_4: BB = BB(0x00000000ff000000);
pub const RANK_5: BB = BB(0x000000ff00000000);
pub const RANK_6: BB = BB(0x0000ff0000000000);
pub const RANK_7: BB = BB(0x00ff000000000000);
pub const RANK_8: BB = BB(0xff00000000000000);

pub const RANK_MASKS: [BB; 8] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

// File Masks
pub const FILE_A: BB = BB(0x0101010101010101);
pub const FILE_B: BB = BB(0x0202020202020202);
pub const FILE_C: BB = BB(0x0404040404040404);
pub const FILE_D: BB = BB(0x0808080808080808);
pub const FILE_E: BB = BB(0x1010101010101010);
pub const FILE_F: BB = BB(0x2020202020202020);
pub const FILE_G: BB = BB(0x4040404040404040);
pub const FILE_H: BB = BB(0x8080808080808080);

pub const FILE_MASKS: [BB; 8] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

pub const FILLED_BB: BB = BB(0xffffffffffffffff);
pub const EMPTY_BB: BB = BB(0x0);

pub const W_QUEENSIDE_ROOK_STARTING_SQ: BB = BB(0x1);
pub const W_KINGSIDE_ROOK_STARTING_SQ: BB = BB(0x80);
pub const B_QUEENSIDE_ROOK_STARTING_SQ: BB = BB(0x100000000000000);
pub const B_KINGSIDE_ROOK_STARTING_SQ: BB = BB(0x8000000000000000);

pub const W_KINGSIDE_CASTLE_TARGET: BB = BB(0x4);
pub const W_QUEENSIDE_CASTLE_TARGET: BB = BB(0x40);
pub const B_KINGSIDE_CASTLE_TARGET: BB = BB(0x400000000000000);
pub const B_QUEENSIDE_CASTLE_TARGET: BB = BB(0x4000000000000000);

pub enum ASCIIBases {
    LowerA = 97,
    UpperA = 65,
    Zero = 48,
}

#[derive(Clone, Copy)]
pub enum Piece {
    Any,
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Piece {
    /// Returns the discriminant of the enum. Use for indexing arrays
    pub fn value(&self) -> usize {
        *self as usize
    }
}

pub enum Color {
    White,
    Black,
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
