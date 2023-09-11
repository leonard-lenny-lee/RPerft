#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
mod bitboard;
mod cache;
#[allow(dead_code)]
mod constants;
mod hash;
mod magics;
mod makemove;
mod movegen;
mod movelist;
#[allow(dead_code)]
mod mv;
#[allow(dead_code)]
pub mod perft;
#[allow(dead_code)]
mod position;
mod tables;
mod types;

use bitboard::BitBoard;

pub use constants::cli::*;
