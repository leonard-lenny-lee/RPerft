use super::*;

use move_::Move;
use types::MoveType;

pub struct MoveList {
    pub moves: Vec<move_::Move>,
    pub n_captures: i32,
    pub n_ep: i32,
    pub n_castles: i32,
    pub n_promotions: i32,
}

impl MoveList {
    pub fn new() -> Self {
        // Based on an average branching factor of 35
        Self {
            moves: Vec::with_capacity(45),
            n_captures: 0,
            n_ep: 0,
            n_castles: 0,
            n_promotions: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    #[inline(always)]
    pub fn add(&mut self, from: BitBoard, to: BitBoard, mt: MoveType) {
        self.moves.push(Move::encode(from, to, mt));
    }

    pub fn add_quiet(&mut self, from: BitBoard, to: BitBoard) {
        self.moves.push(Move::encode(from, to, MoveType::Quiet));
    }

    pub fn add_capture(&mut self, from: BitBoard, to: BitBoard) {
        self.moves.push(Move::encode(from, to, MoveType::Capture));
        self.n_captures += 1;
    }

    pub fn add_ep(&mut self, from: BitBoard, to: BitBoard) {
        self.moves.push(Move::encode(from, to, MoveType::EnPassant));
        self.n_ep += 1;
    }

    pub fn add_castle(&mut self, from: BitBoard, to: BitBoard, mt: MoveType) {
        self.add(from, to, mt);
        self.n_castles += 1;
    }

    pub fn add_promotions(&mut self, from: BitBoard, to: BitBoard) {
        for mt in types::PROMOTION_MOVE_TYPES {
            self.add(from, to, mt);
            self.n_promotions += 1;
        }
    }

    pub fn add_promotion_captures(&mut self, from: BitBoard, to: BitBoard) {
        for mt in types::PROMOTION_CAPTURE_MOVE_TYPES {
            self.add(from, to, mt);
            self.n_promotions += 1;
            self.n_castles += 1;
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.moves.iter()
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        self.moves.index(index)
    }
}
