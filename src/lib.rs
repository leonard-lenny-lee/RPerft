#[macro_use]
extern crate lazy_static;

pub mod bitboard;
pub mod common;
pub mod config;
pub mod makemove;
pub mod movelist;
pub mod position;
pub mod search;
pub mod tables;
pub mod transposition;
pub mod zobrist;
pub mod movegen;
pub mod evaluate;

use bitboard::BB;
use common::*;