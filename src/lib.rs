#[macro_use]
extern crate lazy_static;
extern crate vampirc_uci;

mod bitboard;
mod common;
pub mod engine;
mod evaluate;
mod hash;
mod makemove;
pub mod movegen;
mod movelist;
pub mod position;
mod search;
mod tables;
mod types;
pub mod uci;

use bitboard::BB;
use common::*;
