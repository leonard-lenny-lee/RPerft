/// Contains the internal representation of a chess position
use super::*;
mod analysis;
mod parse;
mod states;
mod zobrist;

use types::{Color, MoveType, PieceType};
use uci::RuntimeError;

pub struct Position {
    pub us: BBSet,
    pub them: BBSet,
    pub occ: BB,
    pub free: BB,
    pub castling_rights: BB,
    pub ep_sq: BB,
    pub halfmove_clock: u8,
    pub fullmove_clock: u8,
    pub key: u64,
    pub wtm: bool,
    pub stm: Color,
    pub ply: u8,
    pub stack: Vec<StackData>,
    pub nnue_pos: NNUEPosition,
}

#[derive(Clone, Copy)]
pub struct BBSet {
    pub all: BB,
    pub pawn: BB,
    pub rook: BB,
    pub knight: BB,
    pub bishop: BB,
    pub queen: BB,
    pub king: BB,
}

#[derive(Clone, Copy)]
pub struct NNUEPosition {
    pub player: usize,
    pub pieces: [usize; 32],
    pub squares: [usize; 32],
    pub board: [usize; 64],
    pub end_ptr: usize,
}

#[derive(Default)]
pub struct StackData {
    // Move info
    pub from: BB,
    pub to: BB,
    pub mt: MoveType,
    // Irretrievable info
    pub moved_pt: PieceType,
    pub captured_pt: Option<PieceType>,
    pub castling_rights: BB,
    pub ep_sq: BB,
    pub halfmove_clock: u8,
    pub key: u64,
}
