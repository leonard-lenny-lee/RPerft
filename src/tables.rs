/// Contains the tables generated at compile time for fast lookups at runtime
use super::*;

mod constants;
mod magics;
mod tables;

pub use magics::initialize;
