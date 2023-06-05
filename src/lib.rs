#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate vampirc_uci as v_uci;

mod bitboard;
#[allow(dead_code)]
mod constants;
mod globals;

pub mod engine;
pub mod evaluate;
pub mod hash;
pub mod makemove;
pub mod movegen;
pub mod movelist;
#[allow(non_camel_case_types)]
pub mod nnue;
pub mod position;
pub mod search;
pub mod types;
pub mod uci;

use bitboard::BB;
use constants::*;
