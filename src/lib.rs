#[macro_use]
extern crate lazy_static;

mod bitboard;
mod cache;
#[allow(dead_code)]
mod constants;
mod hash;
mod magics;
mod makemove;
mod movegen;
mod movelist;
mod mv;
pub mod perft;
mod position;
mod tables;
mod types;

pub mod cli;

use bitboard::BitBoard;
