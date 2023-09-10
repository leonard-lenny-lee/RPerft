/// This file contains hardcoded masks and Enums which are required by many
/// other parts of the program.
use super::*;

pub const MAX_DEPTH: usize = 50;
pub const DEFAULT_TABLE_SIZE_BYTES: usize = 32_000_000;

// Standard chess positions useful for testing
pub mod fen {
    pub const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // Test positions
    pub const TEST_2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    pub const TEST_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    pub const TEST_4: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
    pub const TEST_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    pub const TEST_6: &str =
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ";
}

// Rank Masks
pub mod rank {
    use super::BitBoard;

    pub const RANK_1: BitBoard = BitBoard(0x00000000000000ff);
    pub const RANK_2: BitBoard = BitBoard(0x000000000000ff00);
    pub const RANK_3: BitBoard = BitBoard(0x0000000000ff0000);
    pub const RANK_4: BitBoard = BitBoard(0x00000000ff000000);
    pub const RANK_5: BitBoard = BitBoard(0x000000ff00000000);
    pub const RANK_6: BitBoard = BitBoard(0x0000ff0000000000);
    pub const RANK_7: BitBoard = BitBoard(0x00ff000000000000);
    pub const RANK_8: BitBoard = BitBoard(0xff00000000000000);

    pub const RANK_MASKS: [BitBoard; 8] = [
        RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
    ];
}

// File Masks
pub mod file {
    use super::BitBoard;

    pub const FILE_A: BitBoard = BitBoard(0x0101010101010101);
    pub const FILE_B: BitBoard = BitBoard(0x0202020202020202);
    pub const FILE_C: BitBoard = BitBoard(0x0404040404040404);
    pub const FILE_D: BitBoard = BitBoard(0x0808080808080808);
    pub const FILE_E: BitBoard = BitBoard(0x1010101010101010);
    pub const FILE_F: BitBoard = BitBoard(0x2020202020202020);
    pub const FILE_G: BitBoard = BitBoard(0x4040404040404040);
    pub const FILE_H: BitBoard = BitBoard(0x8080808080808080);

    pub const FILE_MASKS: [BitBoard; 8] = [
        FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
    ];
}

#[allow(dead_code)]
pub mod bb {
    use super::BitBoard;

    pub const A1: BitBoard = BitBoard(1 << 0);
    pub const B1: BitBoard = BitBoard(1 << 1);
    pub const C1: BitBoard = BitBoard(1 << 2);
    pub const D1: BitBoard = BitBoard(1 << 3);
    pub const E1: BitBoard = BitBoard(1 << 4);
    pub const F1: BitBoard = BitBoard(1 << 5);
    pub const G1: BitBoard = BitBoard(1 << 6);
    pub const H1: BitBoard = BitBoard(1 << 7);
    pub const A2: BitBoard = BitBoard(1 << 8);
    pub const B2: BitBoard = BitBoard(1 << 9);
    pub const C2: BitBoard = BitBoard(1 << 10);
    pub const D2: BitBoard = BitBoard(1 << 11);
    pub const E2: BitBoard = BitBoard(1 << 12);
    pub const F2: BitBoard = BitBoard(1 << 13);
    pub const G2: BitBoard = BitBoard(1 << 14);
    pub const H2: BitBoard = BitBoard(1 << 15);
    pub const A3: BitBoard = BitBoard(1 << 16);
    pub const B3: BitBoard = BitBoard(1 << 17);
    pub const C3: BitBoard = BitBoard(1 << 18);
    pub const D3: BitBoard = BitBoard(1 << 19);
    pub const E3: BitBoard = BitBoard(1 << 20);
    pub const F3: BitBoard = BitBoard(1 << 21);
    pub const G3: BitBoard = BitBoard(1 << 22);
    pub const H3: BitBoard = BitBoard(1 << 23);
    pub const A4: BitBoard = BitBoard(1 << 24);
    pub const B4: BitBoard = BitBoard(1 << 25);
    pub const C4: BitBoard = BitBoard(1 << 26);
    pub const D4: BitBoard = BitBoard(1 << 27);
    pub const E4: BitBoard = BitBoard(1 << 28);
    pub const F4: BitBoard = BitBoard(1 << 29);
    pub const G4: BitBoard = BitBoard(1 << 30);
    pub const H4: BitBoard = BitBoard(1 << 31);
    pub const A5: BitBoard = BitBoard(1 << 32);
    pub const B5: BitBoard = BitBoard(1 << 33);
    pub const C5: BitBoard = BitBoard(1 << 34);
    pub const D5: BitBoard = BitBoard(1 << 35);
    pub const E5: BitBoard = BitBoard(1 << 36);
    pub const F5: BitBoard = BitBoard(1 << 37);
    pub const G5: BitBoard = BitBoard(1 << 38);
    pub const H5: BitBoard = BitBoard(1 << 39);
    pub const A6: BitBoard = BitBoard(1 << 40);
    pub const B6: BitBoard = BitBoard(1 << 41);
    pub const C6: BitBoard = BitBoard(1 << 42);
    pub const D6: BitBoard = BitBoard(1 << 43);
    pub const E6: BitBoard = BitBoard(1 << 44);
    pub const F6: BitBoard = BitBoard(1 << 45);
    pub const G6: BitBoard = BitBoard(1 << 46);
    pub const H6: BitBoard = BitBoard(1 << 47);
    pub const A7: BitBoard = BitBoard(1 << 48);
    pub const B7: BitBoard = BitBoard(1 << 49);
    pub const C7: BitBoard = BitBoard(1 << 50);
    pub const D7: BitBoard = BitBoard(1 << 51);
    pub const E7: BitBoard = BitBoard(1 << 52);
    pub const F7: BitBoard = BitBoard(1 << 53);
    pub const G7: BitBoard = BitBoard(1 << 54);
    pub const H7: BitBoard = BitBoard(1 << 55);
    pub const A8: BitBoard = BitBoard(1 << 56);
    pub const B8: BitBoard = BitBoard(1 << 57);
    pub const C8: BitBoard = BitBoard(1 << 58);
    pub const D8: BitBoard = BitBoard(1 << 59);
    pub const E8: BitBoard = BitBoard(1 << 60);
    pub const F8: BitBoard = BitBoard(1 << 61);
    pub const G8: BitBoard = BitBoard(1 << 62);
    pub const H8: BitBoard = BitBoard(1 << 63);

    pub const FULL: BitBoard = BitBoard(0xffffffffffffffff);
    pub const EMPTY: BitBoard = BitBoard(0x0);
}

#[allow(dead_code)]
pub mod ascii {
    pub const LOWER_A: usize = 97;
    pub const UPPER_A: usize = 65;
    pub const ZERO: usize = 48;
}
