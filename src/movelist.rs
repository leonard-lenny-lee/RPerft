use super::*;

use move_::Move;
use types::MoveType;

pub trait MoveList {
    fn add_quiet(&mut self, from: BitBoard, to: BitBoard);
    fn add_double_pawn_push(&mut self, from: BitBoard, to: BitBoard);
    fn add_capture(&mut self, from: BitBoard, to: BitBoard);
    fn add_ep(&mut self, from: BitBoard, to: BitBoard);
    fn add_castle(&mut self, from: BitBoard, to: BitBoard, mt: MoveType);
    fn add_promotions(&mut self, from: BitBoard, to: BitBoard);
    fn add_promotion_captures(&mut self, from: BitBoard, to: BitBoard);
}

pub struct MoveVec(pub Vec<Move>);

impl MoveList for MoveVec {
    fn add_quiet(&mut self, from: BitBoard, to: BitBoard) {
        self.add(from, to, MoveType::Quiet);
    }

    fn add_double_pawn_push(&mut self, from: BitBoard, to: BitBoard) {
        self.add(from, to, MoveType::DoublePawnPush)
    }

    fn add_capture(&mut self, from: BitBoard, to: BitBoard) {
        self.add(from, to, MoveType::Capture);
    }

    fn add_ep(&mut self, from: BitBoard, to: BitBoard) {
        self.add(from, to, MoveType::EnPassant);
    }

    fn add_castle(&mut self, from: BitBoard, to: BitBoard, mt: MoveType) {
        self.add(from, to, mt);
    }

    fn add_promotions(&mut self, from: BitBoard, to: BitBoard) {
        for mt in types::PROMOTION_MOVE_TYPES {
            self.add(from, to, mt);
        }
    }

    fn add_promotion_captures(&mut self, from: BitBoard, to: BitBoard) {
        for mt in types::PROMOTION_CAPTURE_MOVE_TYPES {
            self.add(from, to, mt);
        }
    }
}

impl MoveVec {
    pub fn new() -> Self {
        Self(Vec::with_capacity(45)) // Based on average branching factor of chess
    }

    #[inline(always)]
    fn add(&mut self, from: BitBoard, to: BitBoard, mt: MoveType) {
        self.0.push(Move::encode(from, to, mt));
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.0.iter()
    }
}

impl std::ops::Index<usize> for MoveVec {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

#[derive(Debug, Default)]
pub struct MoveCounter {
    pub count: u32,
    pub captures: u32,
    pub ep: u32,
    pub castles: u32,
    pub promotions: u32,
}

impl MoveList for MoveCounter {
    fn add_quiet(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 1;
    }

    fn add_double_pawn_push(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 1;
    }

    fn add_capture(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 1;
        self.captures += 1;
    }

    fn add_ep(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 1;
        self.ep += 1;
    }

    fn add_castle(&mut self, _from: BitBoard, _to: BitBoard, _mt: MoveType) {
        self.count += 1;
        self.castles += 1;
    }

    fn add_promotions(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 4;
        self.promotions += 4;
    }

    fn add_promotion_captures(&mut self, _from: BitBoard, _to: BitBoard) {
        self.count += 4;
        self.promotions += 4;
        self.captures += 4;
    }
}
