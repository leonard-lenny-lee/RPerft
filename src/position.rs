/// Contains the internal representation of a chess position
use super::*;
use types::{Color, MoveType, PieceType};
use uci::RuntimeError;

mod analysis;
mod parse;
mod states;
mod zobrist;

#[derive(Debug, Clone)]
pub struct Position {
    pub us: BBSet,
    pub them: BBSet,
    pub occupied: BitBoard,
    pub free: BitBoard,
    pub castling_rights: BitBoard,
    pub en_passant: BitBoard,
    pub halfmove_clock: u8,
    pub fullmove_clock: u8,
    pub key: u64,
    pub white_to_move: bool,
    pub side_to_move: Color,
    pub ply: u8, // Ply acts as the stack pointer
    pub stack: Box<[StackData; constants::MAX_DEPTH]>,
    pub nnue_pos: NNUEPosition,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BBSet {
    pub all: BitBoard,
    pub pawn: BitBoard,
    pub rook: BitBoard,
    pub knight: BitBoard,
    pub bishop: BitBoard,
    pub queen: BitBoard,
    pub king: BitBoard,
}

#[derive(Debug, Clone, Copy)]
pub struct NNUEPosition {
    pub player: usize,
    pub pieces: [usize; 32],
    pub squares: [usize; 32],
    pub board: [usize; 64],
    pub end_ptr: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StackData {
    // Move info
    pub from: BitBoard,
    pub to: BitBoard,
    pub mt: MoveType,
    // Irretrievable info
    pub moved_pt: PieceType,
    pub captured_pt: Option<PieceType>,
    pub castling_rights: BitBoard,
    pub en_passant: BitBoard,
    pub halfmove_clock: u8,
    pub key: u64,
    pub restore_index: Option<usize>,
}
