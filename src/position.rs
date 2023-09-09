/// Contains the internal representation of a chess position
use super::*;
use types::Color;

mod analysis;
mod parse;
mod states;
mod zobrist;

#[derive(Debug, Clone, Copy)]
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
    pub ply: u8,
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
