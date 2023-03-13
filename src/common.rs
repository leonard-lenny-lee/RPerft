/// This file contains hardcoded masks and Enums which are required by many
/// other parts of the program.
use super::*;

pub const ENGINE_NAME: &str = "cRusty";
pub const AUTHOR_NAME: &str = "LeonardL";

// Standard chess positions useful for testing
pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
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

pub const A1: BB = BB(1 << 0);
pub const B1: BB = BB(1 << 1);
pub const C1: BB = BB(1 << 2);
pub const D1: BB = BB(1 << 3);
pub const E1: BB = BB(1 << 4);
pub const F1: BB = BB(1 << 5);
pub const G1: BB = BB(1 << 6);
pub const H1: BB = BB(1 << 7);
pub const A2: BB = BB(1 << 8);
pub const B2: BB = BB(1 << 9);
pub const C2: BB = BB(1 << 10);
pub const D2: BB = BB(1 << 11);
pub const E2: BB = BB(1 << 12);
pub const F2: BB = BB(1 << 13);
pub const G2: BB = BB(1 << 14);
pub const H2: BB = BB(1 << 15);
pub const A3: BB = BB(1 << 16);
pub const B3: BB = BB(1 << 17);
pub const C3: BB = BB(1 << 18);
pub const D3: BB = BB(1 << 19);
pub const E3: BB = BB(1 << 20);
pub const F3: BB = BB(1 << 21);
pub const G3: BB = BB(1 << 22);
pub const H3: BB = BB(1 << 23);
pub const A4: BB = BB(1 << 24);
pub const B4: BB = BB(1 << 25);
pub const C4: BB = BB(1 << 26);
pub const D4: BB = BB(1 << 27);
pub const E4: BB = BB(1 << 28);
pub const F4: BB = BB(1 << 29);
pub const G4: BB = BB(1 << 30);
pub const H4: BB = BB(1 << 31);
pub const A5: BB = BB(1 << 32);
pub const B5: BB = BB(1 << 33);
pub const C5: BB = BB(1 << 34);
pub const D5: BB = BB(1 << 35);
pub const E5: BB = BB(1 << 36);
pub const F5: BB = BB(1 << 37);
pub const G5: BB = BB(1 << 38);
pub const H5: BB = BB(1 << 39);
pub const A6: BB = BB(1 << 40);
pub const B6: BB = BB(1 << 41);
pub const C6: BB = BB(1 << 42);
pub const D6: BB = BB(1 << 43);
pub const E6: BB = BB(1 << 44);
pub const F6: BB = BB(1 << 45);
pub const G6: BB = BB(1 << 46);
pub const H6: BB = BB(1 << 47);
pub const A7: BB = BB(1 << 48);
pub const B7: BB = BB(1 << 49);
pub const C7: BB = BB(1 << 50);
pub const D7: BB = BB(1 << 51);
pub const E7: BB = BB(1 << 52);
pub const F7: BB = BB(1 << 53);
pub const G7: BB = BB(1 << 54);
pub const H7: BB = BB(1 << 55);
pub const A8: BB = BB(1 << 56);
pub const B8: BB = BB(1 << 57);
pub const C8: BB = BB(1 << 58);
pub const D8: BB = BB(1 << 59);
pub const E8: BB = BB(1 << 60);
pub const F8: BB = BB(1 << 61);
pub const G8: BB = BB(1 << 62);
pub const H8: BB = BB(1 << 63);

// ASCII Codes for converting int values to char
pub const ASCII_LOWER_A: usize = 97;
pub const ASCII_UPPER_A: usize = 65;
pub const ASCII_ZERO: usize = 48;

pub enum ASCIIBases {
    LowerA = 97,
    UpperA = 65,
    Zero = 48,
}

#[derive(Clone, Copy)]
pub enum Piece {
    All,
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

    /// Returns an iterator that iterates through the pieces only
    pub fn iterpieces() -> std::slice::Iter<'static, Piece> {
        use Piece::*;
        static PIECES: [Piece; 6] = [Pawn, Rook, Knight, Bishop, Queen, King];
        return PIECES.iter();
    }
}

pub enum Axis {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
