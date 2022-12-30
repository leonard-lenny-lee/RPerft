/// Contains the Data struct, which holds the all the bitboards and data
/// to describe the current position, as well as methods to derive other
/// bitboards required for move generation and evaluation

use super::*;
mod states;
mod analysis;
mod data;
mod pieceset;

pub use zobrist::ZobristKey;
pub use data::Data;
use pieceset::PieceSet;
use interface::ExecutionErr;

pub struct Position {
    pub data: Data,
    pub key: ZobristKey,
    state: Box<dyn states::State>
}

impl Position {

    pub fn from_fen(fen: String) -> Result<Self, ExecutionErr> {
        let data = Data::from_fen(fen)?;
        let mut pos = Position::new(&data);
        pos.init_state();
        pos.init_key();
        Ok(pos)
    }

    fn new(data: &Data) -> Self {
        Self {
            data: *data,
            key: ZobristKey(0),
            state: Box::new(states::White)
        }
    }
    
    fn init_state(&mut self) {
        self.state = if self.data.white_to_move {
            Box::new(states::White)
        } else {
            Box::new(states::Black)
        }
    }

    fn init_key(&mut self) {
        self.key = ZobristKey::init_key(&self.data)
    }
    
    pub fn clone(&self) -> Self {
        Self {
            data: self.data,
            key: self.key,
            state: self.state.current_state()
        }
    }
}