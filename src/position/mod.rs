/// Contains the Position struct, which wraps the Data struct, which fully
/// describes a chess position, as well as its Zobrist Hash and state machine
/// which changes the behavior for when it's wtm or btm.
use super::*;
mod analysis;
mod parse;
mod states;
mod zobrist;

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
    pub stm: Color,
    pub ply: u8,
    pub unmake_info: Vec<makemove::UnmakeInfo>,
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
pub enum Color {
    White,
    Black,
}
