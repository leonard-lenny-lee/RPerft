/// Contains the internal representation of a chess position
use super::*;
mod analysis;
mod parse;
mod states;
mod zobrist;

use evaluate::Score;
use types::Color;
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
    pub score: Score,
    pub unmake_info: Vec<makemove::UnmakeInfo>,
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

#[derive(Default)]
pub struct NNUEPosition {
    player: usize,
    pieces: [usize; 32],
    squares: [usize; 32],
    nnue_data: std::collections::VecDeque<nnue::NNUEData>,
}
