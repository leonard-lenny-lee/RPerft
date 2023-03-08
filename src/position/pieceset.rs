use super::*;

#[derive(Clone, Copy)]
pub struct PieceSet {
    pub any: BB,
    pub pawn: BB,
    pub rook: BB,
    pub knight: BB,
    pub bishop: BB,
    pub queen: BB,
    pub king: BB,
}

impl PieceSet {
    pub fn new() -> PieceSet {
        PieceSet {
            any: EMPTY_BB,
            pawn: EMPTY_BB,
            rook: EMPTY_BB,
            knight: EMPTY_BB,
            bishop: EMPTY_BB,
            queen: EMPTY_BB,
            king: EMPTY_BB,
        }
    }

    pub fn as_array(&self) -> [&BB; 7] {
        [
            &self.any,
            &self.pawn,
            &self.rook,
            &self.knight,
            &self.bishop,
            &self.queen,
            &self.king,
        ]
    }

    // Order the bitboards so the index positions are convenient for Zobrist
    // hashing
    pub fn as_hash_array(&self) -> [&BB; 6] {
        [
            &self.pawn,
            &self.knight,
            &self.bishop,
            &self.rook,
            &self.queen,
            &self.king,
        ]
    }

    fn as_mut_array(&mut self) -> [&mut BB; 7] {
        [
            &mut self.any,
            &mut self.pawn,
            &mut self.rook,
            &mut self.knight,
            &mut self.bishop,
            &mut self.queen,
            &mut self.king,
        ]
    }

    pub fn bitor_assign(&mut self, index: usize, rhs: BB) {
        *self.as_mut_array()[index] |= rhs
    }

    pub fn bitxor_assign(&mut self, index: usize, rhs: BB) {
        *self.as_mut_array()[index] ^= rhs;
    }

    pub fn n_kings(&self) -> i16 {
        self.king.pop_count() as i16
    }

    pub fn n_queens(&self) -> i16 {
        self.queen.pop_count() as i16
    }

    pub fn n_rooks(&self) -> i16 {
        self.rook.pop_count() as i16
    }

    pub fn n_bishops(&self) -> i16 {
        self.bishop.pop_count() as i16
    }

    pub fn n_knights(&self) -> i16 {
        self.knight.pop_count() as i16
    }

    pub fn n_pawns(&self) -> i16 {
        self.pawn.pop_count() as i16
    }
}

impl std::ops::Index<usize> for PieceSet {
    type Output = BB;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.any,
            1 => &self.pawn,
            2 => &self.rook,
            3 => &self.knight,
            4 => &self.bishop,
            5 => &self.queen,
            6 => &self.king,
            _ => panic!("Index {} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for PieceSet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.any,
            1 => &mut self.pawn,
            2 => &mut self.rook,
            3 => &mut self.knight,
            4 => &mut self.bishop,
            5 => &mut self.queen,
            6 => &mut self.king,
            _ => panic!("Index {} out of bounds", index),
        }
    }
}
