#[macro_use]
extern crate lazy_static;

pub mod bitboard;
#[allow(dead_code)]
pub mod constants;
pub mod hash;
pub mod makemove;
pub mod move_;
pub mod movegen;
pub mod movelist;
pub mod perft;
pub mod position;
pub mod types;

pub mod cli;

use bitboard::BitBoard;
