/// Contains the Position struct, which holds the all the bitboards and data
/// to describe the current position, as well as methods to derive other
/// bitboards required for move generation and evaluation

mod init;
pub mod analysis_tools;

use super::common::*;

#[derive(Clone, Copy)]
pub struct Position {
    pub w_pieces: PieceSet,
    pub b_pieces: PieceSet,
    pub occ: u64,
    pub free: u64,
    pub white_to_move: bool,
    pub w_kingside_castle: bool,
    pub b_kingside_castle: bool,
    pub w_queenside_castle: bool,
    pub b_queenside_castle: bool,
    pub en_passant_target_sq: u64,
    pub halfmove_clock: i8,
    pub fullmove_clock: i8,
}

#[derive(Clone, Copy)]
pub struct PieceSet {
    pub any: u64,
    pub pawn: u64,
    pub rook: u64,
    pub knight: u64,
    pub bishop: u64,
    pub queen: u64,
    pub king: u64
}

impl PieceSet {
    pub fn new() -> PieceSet {
        PieceSet {
            any: EMPTY_BB,
            pawn: EMPTY_BB,
            rook: EMPTY_BB,
            knight: EMPTY_BB,
            bishop: EMPTY_BB,
            queen: EMPTY_BB,
            king: EMPTY_BB
        }
    }

    pub fn as_array(&self) -> [u64; 7] {
        [self.any, self.pawn, self.rook, self.knight, self.bishop, 
         self.queen, self.king]
    }

    fn as_mut_array(&mut self) -> [&mut u64; 7] {
        [&mut self.any, &mut self.pawn, &mut self.rook, &mut self.knight, 
         &mut self.bishop, &mut self.queen, &mut self.king]
    }

    pub fn bit_or_assign(&mut self, index: usize, rhs: u64) {
        *self.as_mut_array()[index] |= rhs
    }

    pub fn xor_assign(&mut self, index: usize, rhs: u64) {
        *self.as_mut_array()[index] ^= rhs;
    }

}