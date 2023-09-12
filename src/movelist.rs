use super::*;

use std::iter::zip;
use std::ops::{Add, AddAssign};

use mv::Move;
use types::MoveT;

pub trait MoveList {
    // Piece move adders
    fn add_quiets(&mut self, src: BitBoard, targets: BitBoard);
    fn add_captures(&mut self, src: BitBoard, targets: BitBoard);

    // Pawn move adders
    fn add_pawn_pushes(&mut self, srcs: BitBoard, targets: BitBoard);
    fn add_double_pawn_pushes(&mut self, srcs: BitBoard, targets: BitBoard);
    fn add_pawn_captures(&mut self, srcs: BitBoard, targets: BitBoard);
    fn add_ep(&mut self, from: BitBoard, to: BitBoard);
    fn add_castle(&mut self, from: BitBoard, to: BitBoard, mt: MoveT);
    fn add_promos(&mut self, srcs: BitBoard, targets: BitBoard);
    fn add_promo_captures(&mut self, srcs: BitBoard, targets: BitBoard);
}

pub struct MoveVec(pub Vec<Move>);

impl MoveList for MoveVec {
    fn add_quiets(&mut self, src: BitBoard, targets: BitBoard) {
        for to in targets {
            self.add(src, to, MoveT::Quiet);
        }
    }

    fn add_captures(&mut self, src: BitBoard, targets: BitBoard) {
        for to in targets {
            self.add(src, to, MoveT::Capture);
        }
    }

    fn add_pawn_pushes(&mut self, srcs: BitBoard, targets: BitBoard) {
        for (from, to) in zip(srcs, targets) {
            self.add(from, to, MoveT::Quiet);
        }
    }

    fn add_double_pawn_pushes(&mut self, srcs: BitBoard, targets: BitBoard) {
        for (from, to) in zip(srcs, targets) {
            self.add(from, to, MoveT::DoublePawnPush);
        }
    }

    fn add_pawn_captures(&mut self, srcs: BitBoard, targets: BitBoard) {
        for (from, to) in zip(srcs, targets) {
            self.add(from, to, MoveT::Capture);
        }
    }

    fn add_ep(&mut self, from: BitBoard, to: BitBoard) {
        self.add(from, to, MoveT::EnPassant);
    }

    fn add_castle(&mut self, from: BitBoard, to: BitBoard, mt: MoveT) {
        self.add(from, to, mt);
    }

    fn add_promos(&mut self, srcs: BitBoard, targets: BitBoard) {
        for (from, to) in zip(srcs, targets) {
            for mt in types::PROMOTION_MOVE_TYPES {
                self.add(from, to, mt);
            }
        }
    }

    fn add_promo_captures(&mut self, srcs: BitBoard, targets: BitBoard) {
        for (from, to) in zip(srcs, targets) {
            for mt in types::PROMOTION_CAPTURE_MOVE_TYPES {
                self.add(from, to, mt);
            }
        }
    }
}

impl MoveVec {
    pub fn new() -> Self {
        Self(Vec::with_capacity(45)) // Based on average branching factor of chess
    }

    #[inline(always)]
    fn add(&mut self, from: BitBoard, to: BitBoard, mt: MoveT) {
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
    pub nodes: u64,
    pub captures: u32,
    pub ep: u32,
    pub castles: u32,
    pub promotions: u32,
}

impl MoveList for MoveCounter {
    fn add_quiets(&mut self, _src: BitBoard, targets: BitBoard) {
        self.nodes += targets.pop_count() as u64;
    }

    fn add_captures(&mut self, _src: BitBoard, targets: BitBoard) {
        let n = targets.pop_count() as u32;
        self.nodes += n as u64;
        self.captures += n;
    }

    fn add_pawn_pushes(&mut self, _srcs: BitBoard, targets: BitBoard) {
        self.nodes += targets.pop_count() as u64;
    }

    fn add_double_pawn_pushes(&mut self, _srcs: BitBoard, targets: BitBoard) {
        self.nodes += targets.pop_count() as u64;
    }

    fn add_pawn_captures(&mut self, _srcs: BitBoard, targets: BitBoard) {
        let n = targets.pop_count() as u32;
        self.nodes += n as u64;
        self.captures += n;
    }

    fn add_ep(&mut self, _from: BitBoard, _to: BitBoard) {
        self.nodes += 1;
        self.captures += 1;
        self.ep += 1;
    }

    fn add_castle(&mut self, _from: BitBoard, _to: BitBoard, _mt: MoveT) {
        self.nodes += 1;
        self.castles += 1;
    }

    fn add_promos(&mut self, _srcs: BitBoard, targets: BitBoard) {
        let n = targets.pop_count() as u32 * 4;
        self.nodes += n as u64;
        self.promotions += n;
    }

    fn add_promo_captures(&mut self, _srcs: BitBoard, targets: BitBoard) {
        let n = targets.pop_count() as u32 * 4;
        self.nodes += n as u64;
        self.promotions += n;
        self.captures += n;
    }
}

impl AddAssign for MoveCounter {
    fn add_assign(&mut self, rhs: Self) {
        self.nodes += rhs.nodes;
        self.captures += rhs.captures;
        self.ep += rhs.ep;
        self.castles += rhs.castles;
        self.promotions += rhs.promotions;
    }
}

impl Add for MoveCounter {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            nodes: self.nodes + rhs.nodes,
            captures: self.captures + rhs.captures,
            ep: self.ep + rhs.ep,
            castles: self.castles + rhs.castles,
            promotions: self.promotions + rhs.promotions,
        }
    }
}
