use super::*;
use types::Axis;

#[rustfmt::skip]
mod keys;
mod funcs;
mod magics;
mod tables;

#[derive(Debug, Clone, Copy, Default)]
pub struct BitBoard(pub u64);
