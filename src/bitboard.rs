use super::*;
use types::Axis;

#[derive(Debug, Clone, Copy, Default)]
pub struct BB(pub u64);

#[rustfmt::skip]
mod keys;
mod funcs;
mod magics;
mod tables;
