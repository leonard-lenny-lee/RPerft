/// Contains general bitboard manipulation functions
use super::*;
use constants::{ascii, file::*};

impl BitBoard {
    /// Create a one set bit bitboard at a square index
    pub fn from_square(square: usize) -> BitBoard {
        debug_assert!(square < 64);
        BitBoard(1 << square)
    }

    /// Create a bitboard from a vector of square indices
    pub fn from_squares_vec(squares: Vec<usize>) -> BitBoard {
        let mut result = 0;
        for index in squares.iter() {
            result |= 1 << index
        }
        BitBoard(result)
    }

    /// Returns whether the bitboard is empty
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns whether any of the bits are set
    pub fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

    /// Returns how many bits are set on the bitboard
    pub fn pop_count(&self) -> i16 {
        self.0.count_ones() as i16
    }

    /// Return the index of the least significant one bit
    pub fn get_ls1b_index(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    /// Returns the square index of a single bit bitboard
    pub fn to_square(&self) -> usize {
        debug_assert!(self.pop_count() == 1);
        self.0.trailing_zeros() as usize
    }

    /// Return the index of a single bitboard as a u16
    pub fn to_square_uint16(&self) -> u16 {
        debug_assert!(self.pop_count() == 1);
        self.0.trailing_zeros() as u16
    }

    /// Pop off and return the index of the least significant one bit
    pub fn pop_ls1b_index(&mut self) -> usize {
        let ilsb = self.0.trailing_zeros() as usize;
        self.0 ^= 1 << ilsb;
        ilsb
    }

    /// Return a one set bit bitboard of the least significant one bit
    pub fn get_ls1b(&self) -> BitBoard {
        BitBoard(1 << self.0.trailing_zeros())
    }

    /// Pop off and return the least significant one bit as a bitboard
    pub fn pop_ls1b(&mut self) -> BitBoard {
        let lsb = self.get_ls1b();
        *self ^= lsb;
        lsb
    }

    /// Decomposes the bitboard into a vector of one bit bitboards
    pub fn forward_scan(&self) -> Vec<BitBoard> {
        let mut copy = *self;
        let mut out = Vec::new();
        while copy.is_not_empty() {
            out.push(copy.pop_ls1b())
        }
        out
    }

    /// Translate the bitboard north one
    pub const fn north_one(&self) -> BitBoard {
        BitBoard(self.0 << 8)
    }

    /// Translate the bitboard north two
    pub const fn north_two(&self) -> BitBoard {
        BitBoard(self.0 << 16)
    }

    /// Translate the bitboard north east one
    pub const fn nort_east(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_H.0) << 9)
    }

    /// Translate the bitboard east one
    pub const fn east_one(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_H.0) << 1)
    }

    /// Translate the bitboard east two
    pub const fn east_two(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_G.0 | FILE_H.0)) << 2)
    }

    /// Translate the bitboard south east one
    pub const fn sout_east(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_H.0) >> 7)
    }

    /// Translate the bitboard south one
    pub const fn south_one(&self) -> BitBoard {
        BitBoard(self.0 >> 8)
    }

    /// Translate the bitboard south two
    pub const fn south_two(&self) -> BitBoard {
        BitBoard(self.0 >> 16)
    }

    /// Translate the bitboard south west one
    pub const fn sout_west(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_A.0) >> 9)
    }

    /// Translate the bitboard west one
    pub const fn west_one(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_A.0) >> 1)
    }

    /// Translate the bitboard west two
    pub const fn west_two(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_A.0 | FILE_B.0)) >> 2)
    }

    /// Translate the bitboard north west one
    pub const fn nort_west(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_A.0) << 7)
    }

    /// Translate the bitboard north north east
    pub const fn no_no_ea(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_H.0) << 17)
    }

    /// Translate the bitboard east east
    pub const fn no_ea_ea(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_G.0 | FILE_H.0)) << 10)
    }

    /// Translate the bitboard south east east
    pub const fn so_ea_ea(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_G.0 | FILE_H.0)) >> 6)
    }

    /// Translate the bitboard south south east
    pub const fn so_so_ea(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_H.0) >> 15)
    }

    /// Translate the bitboard south south west
    pub const fn so_so_we(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_A.0) >> 17)
    }

    /// Translate the bitboard south west west
    pub const fn so_we_we(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_A.0 | FILE_B.0)) >> 10)
    }

    /// Translate the bitboard north west west
    pub const fn no_we_we(&self) -> BitBoard {
        BitBoard((self.0 & !(FILE_A.0 | FILE_B.0)) << 6)
    }

    /// Translate the bitboard north north west
    pub const fn no_no_we(&self) -> BitBoard {
        BitBoard((self.0 & !FILE_A.0) << 15)
    }

    /// Flip the bitboard in the vertical direction
    pub fn flip_vertical(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
    }

    /// Reverse the endianness of the bitboard
    pub fn reverse_bits(&self) -> BitBoard {
        BitBoard(self.0.reverse_bits())
    }

    /// Return the attack squares of the knight
    /// * Uses bitwise shifting for compile time generation of lookup tables.
    /// * Use lookup_knight_attacks for run time.
    pub const fn generate_knight_attacks(&self) -> BitBoard {
        BitBoard(
            self.no_no_ea().0
                | self.no_ea_ea().0
                | self.so_ea_ea().0
                | self.so_so_ea().0
                | self.so_so_we().0
                | self.so_we_we().0
                | self.no_we_we().0
                | self.no_no_we().0,
        )
    }

    /// Return the attack squares of the king
    /// * Uses bitwise shifting for compile time generation of lookup tables.
    /// * Use lookup_king_attacks for run time.
    pub const fn generate_king_attacks(&self) -> BitBoard {
        BitBoard(
            self.north_one().0
                | self.nort_east().0
                | self.east_one().0
                | self.sout_east().0
                | self.south_one().0
                | self.sout_west().0
                | self.west_one().0
                | self.nort_west().0,
        )
    }

    /// Kogge-Stone north fill
    pub const fn ks_nort_fill(&self) -> BitBoard {
        let mut out = self.0;
        out |= out << 8;
        out |= out << 16;
        out |= out << 32;
        return BitBoard(out);
    }

    /// Kogge-Stone south fill
    pub const fn ks_sout_fill(&self) -> BitBoard {
        let mut out = self.0;
        out |= out >> 8;
        out |= out >> 16;
        out |= out >> 32;
        return BitBoard(out);
    }

    /// Kogge-Stone east fill
    pub const fn ks_east_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 << 1);
        let m_3 = m_2 & (m_2 << 2);
        out |= m_1 & (out << 1);
        out |= m_2 & (out << 2);
        out |= m_3 & (out << 4);
        return BitBoard(out);
    }

    /// Kogge-Stone north east fill
    pub const fn ks_no_ea_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 << 9);
        let m_3 = m_2 & (m_2 << 18);
        out |= m_1 & (out << 9);
        out |= m_2 & (out << 18);
        out |= m_3 & (out << 36);
        return BitBoard(out);
    }

    /// Kogge-Stone south east fill
    pub const fn ks_so_ea_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 >> 7);
        let m_3 = m_2 & (m_2 >> 14);
        out |= m_1 & (out >> 7);
        out |= m_2 & (out >> 14);
        out |= m_3 & (out >> 28);
        return BitBoard(out);
    }

    /// Kogge-Stone west fill
    pub const fn ks_west_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 >> 1);
        let m_3 = m_2 & (m_2 >> 2);
        out |= m_1 & (out >> 1);
        out |= m_2 & (out >> 2);
        out |= m_3 & (out >> 4);
        return BitBoard(out);
    }

    /// Kogge-Stone south west fill
    pub const fn ks_so_we_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 >> 9);
        let m_3 = m_2 & (m_2 >> 18);
        out |= m_1 & (out >> 9);
        out |= m_2 & (out >> 18);
        out |= m_3 & (out >> 36);
        return BitBoard(out);
    }

    /// Kogge-Stone north west fill
    pub const fn ks_no_we_fill(&self) -> BitBoard {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 << 7);
        let m_3 = m_2 & (m_2 << 14);
        out |= m_1 & (out << 7);
        out |= m_2 & (out << 14);
        out |= m_3 & (out << 28);
        return BitBoard(out);
    }

    /// Kogge-Stone occluded north fill
    pub const fn ks_nort_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_1 |= bb_2 & (bb_1 << 8);
        bb_2 &= bb_2 << 8;
        bb_1 |= bb_2 & (bb_1 << 16);
        bb_2 &= bb_2 << 16;
        bb_1 |= bb_2 & (bb_1 << 32);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded south fill
    pub const fn ks_sout_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_1 |= bb_2 & (bb_1 >> 8);
        bb_2 &= bb_2 >> 8;
        bb_1 |= bb_2 & (bb_1 >> 16);
        bb_2 &= bb_2 >> 16;
        bb_1 |= bb_2 & (bb_1 >> 32);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded east fill
    pub const fn ks_east_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 << 1);
        bb_2 &= bb_2 << 1;
        bb_1 |= bb_2 & (bb_1 << 2);
        bb_2 &= bb_2 << 2;
        bb_1 |= bb_2 & (bb_1 << 4);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded west fill
    pub const fn ks_west_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 >> 1);
        bb_2 &= bb_2 >> 1;
        bb_1 |= bb_2 & (bb_1 >> 2);
        bb_2 &= bb_2 >> 2;
        bb_1 |= bb_2 & (bb_1 >> 4);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded north east fill
    pub const fn ks_no_ea_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 << 9);
        bb_2 &= bb_2 << 9;
        bb_1 |= bb_2 & (bb_1 << 18);
        bb_2 &= bb_2 << 18;
        bb_1 |= bb_2 & (bb_1 << 36);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded south east fill
    pub const fn ks_so_ea_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 >> 7);
        bb_2 &= bb_2 >> 7;
        bb_1 |= bb_2 & (bb_1 >> 14);
        bb_2 &= bb_2 >> 14;
        bb_1 |= bb_2 & (bb_1 >> 28);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded north west fill
    pub const fn ks_no_we_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 << 7);
        bb_2 &= bb_2 << 7;
        bb_1 |= bb_2 & (bb_1 << 14);
        bb_2 &= bb_2 << 14;
        bb_1 |= bb_2 & (bb_1 << 28);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone occluded south west fill
    pub const fn ks_so_we_ofill(&self, other: BitBoard) -> BitBoard {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 >> 9);
        bb_2 &= bb_2 >> 9;
        bb_1 |= bb_2 & (bb_1 >> 18);
        bb_2 &= bb_2 >> 18;
        bb_1 |= bb_2 & (bb_1 >> 36);
        return BitBoard(bb_1);
    }

    /// Kogge-Stone north attack squares
    pub fn ks_north_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_nort_ofill(other).north_one()
    }

    /// Kogge-Stone south attack squares
    pub fn ks_south_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_sout_ofill(other).south_one()
    }

    /// Kogge-Stone east attack squares
    pub fn ks_east_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_east_ofill(other).east_one()
    }

    /// Kogge-Stone west attack squares
    pub fn ks_west_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_west_ofill(other).west_one()
    }

    /// Kogge-Stone north-east attack squares
    pub fn ks_no_ea_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_no_ea_ofill(other).nort_east()
    }

    /// Kogge-Stone north-west attack squares
    pub fn ks_no_we_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_no_we_ofill(other).nort_west()
    }

    /// Kogge-Stone south-east attack squares
    pub fn ks_so_ea_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_so_ea_ofill(other).sout_east()
    }

    /// Kogge-Stone south-west attack squares
    pub fn ks_so_we_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_so_we_ofill(other).sout_west()
    }

    /// Kogge-Stone file attack squares
    pub fn ks_file_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_north_attacks(other) | self.ks_south_attacks(other)
    }

    /// Kogge-Stone rank attack squares
    pub fn ks_rank_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_east_attacks(other) | self.ks_west_attacks(other)
    }

    /// Kogge-Stone diagonal attack squares
    pub fn ks_diag_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_no_ea_attacks(other) | self.ks_so_we_attacks(other)
    }

    /// Kogge-Stone anti-diagonal attack squares
    pub fn ks_adiag_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_no_we_attacks(other) | self.ks_so_ea_attacks(other)
    }

    /// Kogge-Stone rook attack squares
    pub fn ks_rook_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_file_attacks(other) | self.ks_rank_attacks(other)
    }

    /// Kogge-Stone bishop attack squares
    pub fn ks_bishop_attacks(&self, other: BitBoard) -> BitBoard {
        self.ks_diag_attacks(other) | self.ks_adiag_attacks(other)
    }

    /// Uses hyperbola quintessence to find the attack squares of a single
    /// rook, taking into account the occupancy of the current board
    pub fn hyp_quint_rook_attacks(&self, occ: BitBoard) -> BitBoard {
        self.hyp_quint(occ, Axis::File) | self.hyp_quint(occ, Axis::Rank)
    }

    /// Uses hyperbola quintessence to find the attack squares of a single
    /// bishop, taking into account the occupancy of the current board
    pub fn hyp_quint_bishop_attacks(&self, occ: BitBoard) -> BitBoard {
        self.hyp_quint(occ, Axis::Diagonal) | self.hyp_quint(occ, Axis::AntiDiagonal)
    }

    /// Return a bitboard of the common axis shared between this single bit
    /// bitboard and another single bit bitboard
    pub fn between_mask(&self, other: BitBoard) -> BitBoard {
        debug_assert_eq!(self.pop_count(), 1);
        debug_assert_eq!(other.pop_count(), 1);
        debug_assert_ne!(*self, other);

        for ax in self.lookup_axes_array() {
            if (ax & other).is_not_empty() {
                return ax;
            }
        }

        panic!(
            "no axis between {} and {}",
            self.to_square(),
            other.to_square()
        )
    }

    /// Convert from algebraic notation e.g. a5 to a one bit bitboard
    pub fn from_algebraic(algebraic: &str) -> Result<BitBoard, uci::RuntimeError> {
        let chars: Vec<char> = algebraic.chars().collect();
        if chars.len() != 2 {
            return Err(uci::RuntimeError::AlgebraicParseError(
                algebraic.to_string(),
            ));
        }
        if !chars[0].is_alphabetic() || !chars[1].is_numeric() {
            return Err(uci::RuntimeError::AlgebraicParseError(
                algebraic.to_string(),
            ));
        }
        let file = chars[0].to_ascii_lowercase() as u8 - ascii::LOWER_A as u8;
        let rank = chars[1] as u8 - ascii::ZERO as u8;
        if rank <= 8 && file <= 8 {
            let square_index = file + (rank - 1) * 8;
            Ok(BitBoard(1 << square_index))
        } else {
            Err(uci::RuntimeError::AlgebraicParseError(
                algebraic.to_string(),
            ))
        }
    }

    /// Convert a one bit bitboard into algebraic notation
    pub fn to_algebraic(&self) -> String {
        assert!(
            self.0.count_ones() == 1,
            "Attempted algebraic conversion on bitboard with greater than one bit"
        );
        let index = self.get_ls1b_index();
        let rank_index = (index / 8) as u8;
        let file_index = (index % 8) as u8;
        let rank = (ascii::ZERO as u8 + rank_index + 1) as char;
        let file = (ascii::LOWER_A as u8 + file_index) as char;
        format!("{file}{rank}")
    }

    /// Convert to a string representation for printing to the standard output
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("   --- --- --- --- --- --- --- --- \n8 ");
        for i in 0..64 {
            if i % 8 == 0 && i != 0 {
                let rank = &(8 - (i / 8)).to_string()[..];
                out.push_str("|\n   --- --- --- --- --- --- --- --- \n");
                out.push_str(rank);
                out.push(' ')
            }
            if ((1 << (7 - i / 8) * 8 + (i % 8)) & self.0) != 0 {
                out.push_str("| x ")
            } else {
                out.push_str("|   ")
            }
        }
        out.push_str("|\n   --- --- --- --- --- --- --- ---");
        out.push_str(" \n    a   b   c   d   e   f   g   h ");
        out
    }

    /// Convert to the UciSquare struct in the vampirc_uci crate
    pub fn to_uci_square(&self) -> v_uci::UciSquare {
        debug_assert_eq!(self.pop_count(), 1);

        let sq = self.to_square() as u8;
        let file = (sq % 8 + ascii::LOWER_A as u8) as char;
        let rank = (sq / 8 + 1) as u8;

        return v_uci::UciSquare { file, rank };
    }
}

impl std::cmp::PartialEq for BitBoard {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}

impl std::cmp::Eq for BitBoard {}

impl std::ops::BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl std::ops::Mul<u64> for BitBoard {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        BitBoard(self.0.wrapping_mul(rhs))
    }
}

impl std::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::ops::Shl<usize> for BitBoard {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl std::ops::Shr<usize> for BitBoard {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl std::ops::Sub for BitBoard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BitBoard(self.0.wrapping_sub(rhs.0))
    }
}

impl std::ops::SubAssign for BitBoard {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0)
    }
}

impl std::iter::Iterator for BitBoard {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == constants::bb::EMPTY.0 {
            return None;
        }
        let lsb = 1 << self.0.trailing_zeros();
        self.0 ^= lsb;
        return Some(BitBoard(lsb));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::{bb, rank};

    use test_case::test_case;

    #[test]
    fn test_bitmask_to_algebraic() {
        let expected = "f8";
        let input = BitBoard::from_square(61);
        let result = &input.to_algebraic()[..];
        assert_eq!(expected, result)
    }

    #[test]
    fn test_squares_to_bitboard() {
        let squares = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let bitboard = BitBoard::from_squares_vec(squares);
        assert_eq!(bitboard, rank::RANK_1);
    }

    #[test]
    fn test_get_ls1b() {
        let bb = BitBoard::from_squares_vec(vec![19, 30]);
        let lsb = bb.get_ls1b();
        assert_eq!(BitBoard::from_square(19), lsb);
    }

    #[test]
    fn test_get_ilsb() {
        let bitboard = BitBoard::from_squares_vec(vec![41, 55]);
        let ilsb = bitboard.get_ls1b_index();
        assert_eq!(ilsb, 41);
    }

    #[test]
    fn test_forward_scan() {
        let scan_result = FILE_E.forward_scan();
        let expected = vec![
            BitBoard::from_square(4),
            BitBoard::from_square(12),
            BitBoard::from_square(20),
            BitBoard::from_square(28),
            BitBoard::from_square(36),
            BitBoard::from_square(44),
            BitBoard::from_square(52),
            BitBoard::from_square(60),
        ];
        assert_eq!(scan_result, expected);
    }

    #[test_case(Axis::Rank, vec![17, 19, 23, 35], 19, vec![17, 18, 20, 21, 22, 23];"RANK")]
    #[test_case(Axis::File, vec![20, 44, 18], 20, vec![4, 12, 28, 36, 44];"FILE")]
    #[test_case(Axis::Diagonal, vec![27, 54, 18], 27, vec![18, 36, 45, 54];"DIAG")]
    #[test_case(Axis::AntiDiagonal, vec![6, 13, 34, 41, 43], 34, vec![41, 27, 20, 13];"ADIAG")]
    fn test_hyp_quint(axis: Axis, occ: Vec<usize>, slider: usize, expected: Vec<usize>) {
        let occ = BitBoard::from_squares_vec(occ);
        let slider = BitBoard::from_square(slider);
        let result = slider.hyp_quint(occ, axis);
        let expected = BitBoard::from_squares_vec(expected);
        assert_eq!(result, expected);
    }

    #[ignore]
    #[test]
    fn test_print_bb() {
        let king = bb::E1;
        let checker = bb::E8;
        let out = king.between_bb(checker).to_string();
        // let conn = king.connect_squares(checker);
        // let out = conn.to_string();
        print!("{}", out)
    }

    #[test]
    fn test_flip_vertical() {
        let bb = BitBoard(0x8040201);
        let result = bb.flip_vertical();
        assert_eq!(result, BitBoard(0x102040800000000))
    }

    #[test]
    fn test_to_uci_sq() {
        let result = bb::F5.to_uci_square();
        assert_eq!(result.file, 'f');
        assert_eq!(result.rank, 5)
    }
}
