#[macro_use]
extern crate lazy_static;
extern crate vampirc_uci;

pub mod bitboard;
pub mod common;
pub mod config;
pub mod engine;
pub mod evaluate;
pub mod hash;
pub mod makemove;
pub mod movegen;
pub mod movelist;
pub mod position;
pub mod search;
pub mod tables;
pub mod uci;
pub mod zobrist;

use bitboard::BB;
use common::*;
