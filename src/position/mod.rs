/// Contains the internal representation of a chess position
use super::*;
use types::Color;

mod analysis;
mod parse;
mod states;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub us: BitBoardSet,
    pub them: BitBoardSet,
    pub occ: BitBoard,
    pub free: BitBoard,
    pub castling_rights: BitBoard,
    pub ep_sq: BitBoard,
    pub halfmove_clock: u8,
    pub fullmove_clock: u8,
    pub key: u64,
    pub wtm: bool,
    pub stm: Color,
    pub ply: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BitBoardSet {
    pub all: BitBoard,
    pub pawn: BitBoard,
    pub rook: BitBoard,
    pub knight: BitBoard,
    pub bishop: BitBoard,
    pub queen: BitBoard,
    pub king: BitBoard,
}
