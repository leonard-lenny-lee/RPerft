#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate vampirc_uci as v_uci;

mod bitboard;
#[allow(dead_code)]
mod constants;
mod evaluate;
mod globals;
mod hash;
mod makemove;
mod movegen;
mod movelist;
#[allow(non_camel_case_types, dead_code)]
mod nnue;
mod position;
mod search;
mod types;

pub mod engine;
pub mod uci;

use bitboard::BitBoard;
