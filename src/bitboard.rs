use super::*;
use types::Axis;

#[derive(Debug, Clone, Copy)]
pub struct BB(pub u64);

impl BB {
    /// Create a one set bit bitboard at an index
    pub fn from_index(index: usize) -> BB {
        BB(1 << index)
    }

    /// Create a bitboard from a vector of bitboard indices
    pub fn from_indices(indices: Vec<usize>) -> BB {
        let mut result = 0;
        for index in indices.iter() {
            result |= 1 << index
        }
        BB(result)
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
    pub fn ils1b(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    /// Returns the index of a single bit bitboard
    pub fn to_index(&self) -> usize {
        debug_assert!(self.pop_count() == 1);
        self.0.trailing_zeros() as usize
    }

    /// Return the index of a single bitboard as a u16
    pub fn to_index_uint16(&self) -> u16 {
        debug_assert!(self.pop_count() == 1);
        self.0.trailing_zeros() as u16
    }

    /// Pop off and return the index of the least significant one bit
    pub fn pop_ils1b(&mut self) -> usize {
        let ilsb = self.0.trailing_zeros() as usize;
        self.0 ^= 1 << ilsb;
        return ilsb;
    }

    /// Return a one set bit bitboard of the least significant one bit
    pub fn ls1b(&self) -> BB {
        BB(1 << self.0.trailing_zeros())
    }

    /// Pop off and return the least significant one bit as a bitboard
    pub fn pop_ls1b(&mut self) -> BB {
        let lsb = self.ls1b();
        *self ^= lsb;
        return lsb;
    }

    /// Decomposes the bitboard into a vector of one bit bitboards
    pub fn forward_scan(&self) -> Vec<BB> {
        let mut copy_self = *self;
        let mut scan_result = Vec::new();
        while copy_self.is_not_empty() {
            scan_result.push(copy_self.pop_ls1b())
        }
        return scan_result;
    }

    /// Translate the bitboard north one
    pub const fn north_one(&self) -> BB {
        BB(self.0 << 8)
    }

    /// Translate the bitboard north two
    pub const fn north_two(&self) -> BB {
        BB(self.0 << 16)
    }

    /// Translate the bitboard north east one
    pub const fn nort_east(&self) -> BB {
        BB((self.0 & !FILE_H.0) << 9)
    }

    /// Translate the bitboard east one
    pub const fn east_one(&self) -> BB {
        BB((self.0 & !FILE_H.0) << 1)
    }

    /// Translate the bitboard east two
    pub const fn east_two(&self) -> BB {
        BB((self.0 & !(FILE_G.0 | FILE_H.0)) << 2)
    }

    /// Translate the bitboard south east one
    pub const fn sout_east(&self) -> BB {
        BB((self.0 & !FILE_H.0) >> 7)
    }

    /// Translate the bitboard south one
    pub const fn south_one(&self) -> BB {
        BB(self.0 >> 8)
    }

    /// Translate the bitboard south two
    pub const fn south_two(&self) -> BB {
        BB(self.0 >> 16)
    }

    /// Translate the bitboard south west one
    pub const fn sout_west(&self) -> BB {
        BB((self.0 & !FILE_A.0) >> 9)
    }

    /// Translate the bitboard west one
    pub const fn west_one(&self) -> BB {
        BB((self.0 & !FILE_A.0) >> 1)
    }

    /// Translate the bitboard west two
    pub const fn west_two(&self) -> BB {
        BB((self.0 & !(FILE_A.0 | FILE_B.0)) >> 2)
    }

    /// Translate the bitboard north west one
    pub const fn nort_west(&self) -> BB {
        BB((self.0 & !FILE_A.0) << 7)
    }

    /// Translate the bitboard north north east
    pub const fn no_no_ea(&self) -> BB {
        BB((self.0 & !FILE_H.0) << 17)
    }

    /// Translate the bitboard east east
    pub const fn no_ea_ea(&self) -> BB {
        BB((self.0 & !(FILE_G.0 | FILE_H.0)) << 10)
    }

    /// Translate the bitboard south east east
    pub const fn so_ea_ea(&self) -> BB {
        BB((self.0 & !(FILE_G.0 | FILE_H.0)) >> 6)
    }

    /// Translate the bitboard south south east
    pub const fn so_so_ea(&self) -> BB {
        BB((self.0 & !FILE_H.0) >> 15)
    }

    /// Translate the bitboard south south west
    pub const fn so_so_we(&self) -> BB {
        BB((self.0 & !FILE_A.0) >> 17)
    }

    /// Translate the bitboard south west west
    pub const fn so_we_we(&self) -> BB {
        BB((self.0 & !(FILE_A.0 | FILE_B.0)) >> 10)
    }

    /// Translate the bitboard north west west
    pub const fn no_we_we(&self) -> BB {
        BB((self.0 & !(FILE_A.0 | FILE_B.0)) << 6)
    }

    /// Translate the bitboard north north west
    pub const fn no_no_we(&self) -> BB {
        BB((self.0 & !FILE_A.0) << 15)
    }

    /// Flip the bitboard in the vertical direction
    pub fn flip_vertical(&self) -> BB {
        BB(self.0.swap_bytes())
    }

    /// Reverse the endianness of the bitboard
    pub fn reverse_bits(&self) -> BB {
        BB(self.0.reverse_bits())
    }

    /// Return the attack squares of the knight
    /// * Uses bitwise shifting for compile time generation of lookup tables.
    /// * Use lookup_knight_attacks for run time.
    pub const fn knight_attack_squares(&self) -> BB {
        BB(self.no_no_ea().0
            | self.no_ea_ea().0
            | self.so_ea_ea().0
            | self.so_so_ea().0
            | self.so_so_we().0
            | self.so_we_we().0
            | self.no_we_we().0
            | self.no_no_we().0)
    }

    /// Return the attack squares of the king
    /// * Uses bitwise shifting for compile time generation of lookup tables.
    /// * Use lookup_king_attacks for run time.
    pub const fn king_attack_squares(&self) -> BB {
        BB(self.north_one().0
            | self.nort_east().0
            | self.east_one().0
            | self.sout_east().0
            | self.south_one().0
            | self.sout_west().0
            | self.west_one().0
            | self.nort_west().0)
    }

    /// Kogge-Stone north fill
    pub const fn nort_fill(&self) -> BB {
        let mut out = self.0;
        out |= out << 8;
        out |= out << 16;
        out |= out << 32;
        return BB(out);
    }

    /// Kogge-Stone south fill
    pub const fn sout_fill(&self) -> BB {
        let mut out = self.0;
        out |= out >> 8;
        out |= out >> 16;
        out |= out >> 32;
        return BB(out);
    }

    /// Kogge-Stone east fill
    pub const fn east_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 << 1);
        let m_3 = m_2 & (m_2 << 2);
        out |= m_1 & (out << 1);
        out |= m_2 & (out << 2);
        out |= m_3 & (out << 4);
        return BB(out);
    }

    /// Kogge-Stone north east fill
    pub const fn no_ea_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 << 9);
        let m_3 = m_2 & (m_2 << 18);
        out |= m_1 & (out << 9);
        out |= m_2 & (out << 18);
        out |= m_3 & (out << 36);
        return BB(out);
    }

    /// Kogge-Stone south east fill
    pub const fn so_ea_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_A.0;
        let m_2 = m_1 & (m_1 >> 7);
        let m_3 = m_2 & (m_2 >> 14);
        out |= m_1 & (out >> 7);
        out |= m_2 & (out >> 14);
        out |= m_3 & (out >> 28);
        return BB(out);
    }

    /// Kogge-Stone west fill
    pub const fn west_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 >> 1);
        let m_3 = m_2 & (m_2 >> 2);
        out |= m_1 & (out >> 1);
        out |= m_2 & (out >> 2);
        out |= m_3 & (out >> 4);
        return BB(out);
    }

    /// Kogge-Stone south west fill
    pub const fn so_we_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 >> 9);
        let m_3 = m_2 & (m_2 >> 18);
        out |= m_1 & (out >> 9);
        out |= m_2 & (out >> 18);
        out |= m_3 & (out >> 36);
        return BB(out);
    }

    /// Kogge-Stone north west fill
    pub const fn no_we_fill(&self) -> BB {
        let mut out = self.0;
        let m_1 = !FILE_H.0;
        let m_2 = m_1 & (m_1 << 7);
        let m_3 = m_2 & (m_2 << 14);
        out |= m_1 & (out << 7);
        out |= m_2 & (out << 14);
        out |= m_3 & (out << 28);
        return BB(out);
    }

    /// Kogge-Stone occluded north fill
    pub const fn nort_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_1 |= bb_2 & (bb_1 << 8);
        bb_2 &= bb_2 << 8;
        bb_1 |= bb_2 & (bb_1 << 16);
        bb_2 &= bb_2 << 16;
        bb_1 |= bb_2 & (bb_1 << 32);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded south fill
    pub const fn sout_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_1 |= bb_2 & (bb_1 >> 8);
        bb_2 &= bb_2 >> 8;
        bb_1 |= bb_2 & (bb_1 >> 16);
        bb_2 &= bb_2 >> 16;
        bb_1 |= bb_2 & (bb_1 >> 32);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded east fill
    pub const fn east_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 << 1);
        bb_2 &= bb_2 << 1;
        bb_1 |= bb_2 & (bb_1 << 2);
        bb_2 &= bb_2 << 2;
        bb_1 |= bb_2 & (bb_1 << 4);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded west fill
    pub const fn west_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 >> 1);
        bb_2 &= bb_2 >> 1;
        bb_1 |= bb_2 & (bb_1 >> 2);
        bb_2 &= bb_2 >> 2;
        bb_1 |= bb_2 & (bb_1 >> 4);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded north east fill
    pub const fn no_ea_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 << 9);
        bb_2 &= bb_2 << 9;
        bb_1 |= bb_2 & (bb_1 << 18);
        bb_2 &= bb_2 << 18;
        bb_1 |= bb_2 & (bb_1 << 36);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded south east fill
    pub const fn so_ea_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_A.0;
        bb_1 |= bb_2 & (bb_1 >> 7);
        bb_2 &= bb_2 >> 7;
        bb_1 |= bb_2 & (bb_1 >> 14);
        bb_2 &= bb_2 >> 14;
        bb_1 |= bb_2 & (bb_1 >> 28);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded north west fill
    pub const fn no_we_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 << 7);
        bb_2 &= bb_2 << 7;
        bb_1 |= bb_2 & (bb_1 << 14);
        bb_2 &= bb_2 << 14;
        bb_1 |= bb_2 & (bb_1 << 28);
        return BB(bb_1);
    }

    /// Kogge-Stone occluded south west fill
    pub const fn so_we_ofill(&self, other: BB) -> BB {
        let (mut bb_1, mut bb_2) = (self.0, !other.0);
        bb_2 &= !FILE_H.0;
        bb_1 |= bb_2 & (bb_1 >> 9);
        bb_2 &= bb_2 >> 9;
        bb_1 |= bb_2 & (bb_1 >> 18);
        bb_2 &= bb_2 >> 18;
        bb_1 |= bb_2 & (bb_1 >> 36);
        return BB(bb_1);
    }

    /// Kogge-Stone north attack squares
    pub fn nort_attacks(&self, other: BB) -> BB {
        self.nort_ofill(other).north_one()
    }

    /// Kogge-Stone south attack squares
    pub fn sout_attacks(&self, other: BB) -> BB {
        self.sout_ofill(other).south_one()
    }

    /// Kogge-Stone east attack squares
    pub fn east_attacks(&self, other: BB) -> BB {
        self.east_ofill(other).east_one()
    }

    /// Kogge-Stone west attack squares
    pub fn west_attacks(&self, other: BB) -> BB {
        self.west_ofill(other).west_one()
    }

    /// Kogge-Stone north-east attack squares
    pub fn no_ea_attacks(&self, other: BB) -> BB {
        self.no_ea_ofill(other).nort_east()
    }

    /// Kogge-Stone north-west attack squares
    pub fn no_we_attacks(&self, other: BB) -> BB {
        self.no_we_ofill(other).nort_west()
    }

    /// Kogge-Stone south-east attack squares
    pub fn so_ea_attacks(&self, other: BB) -> BB {
        self.so_ea_ofill(other).sout_east()
    }

    /// Kogge-Stone south-west attack squares
    pub fn so_we_attacks(&self, other: BB) -> BB {
        self.so_we_ofill(other).sout_west()
    }

    /// Kogge-Stone file attack squares
    pub fn file_attacks(&self, other: BB) -> BB {
        self.nort_attacks(other) | self.sout_attacks(other)
    }

    /// Kogge-Stone rank attack squares
    pub fn rank_attacks(&self, other: BB) -> BB {
        self.east_attacks(other) | self.west_attacks(other)
    }

    /// Kogge-Stone diagonal attack squares
    pub fn diag_attacks(&self, other: BB) -> BB {
        self.no_ea_attacks(other) | self.so_we_attacks(other)
    }

    /// Kogge-Stone anti-diagonal attack squares
    pub fn adiag_attacks(&self, other: BB) -> BB {
        self.no_we_attacks(other) | self.so_ea_attacks(other)
    }

    /// Kogge-Stone rook attack squares
    pub fn rook_attacks(&self, other: BB) -> BB {
        self.file_attacks(other) | self.rank_attacks(other)
    }

    /// Kogge-Stone bishop attack squares
    pub fn bishop_attacks(&self, other: BB) -> BB {
        self.diag_attacks(other) | self.adiag_attacks(other)
    }

    /// Uses hyperbola quintessence to find the attack squares of a single
    /// rook, taking into account the occupancy of the current board
    pub fn rook_hq(&self, occ: BB) -> BB {
        self.hyp_quint(occ, Axis::File) | self.hyp_quint(occ, Axis::Rank)
    }

    /// Uses hyperbola quintessence to find the attack squares of a single
    /// bishop, taking into account the occupancy of the current board
    pub fn bishop_hq(&self, occ: BB) -> BB {
        self.hyp_quint(occ, Axis::Diagonal) | self.hyp_quint(occ, Axis::AntiDiagonal)
    }

    /// Return a bitboard with the intervening bits between this single bit
    /// bitboard and another single bit bitboard filled
    pub fn connect_squares(&self, other: BB) -> BB {
        debug_assert_eq!(self.0.count_ones(), 1);
        debug_assert_eq!(other.0.count_ones(), 1);
        debug_assert_ne!(*self, other);
        // Calculate if the bitboards are connected via a file/rank or
        // a diagonal/antidiagonal
        let (this_sq, other_sq) = (self.to_index(), other.to_index());
        if this_sq / 8 == other_sq / 8 || this_sq % 8 == other_sq % 8 {
            self.lu_rook_attacks(other) & other.lu_rook_attacks(*self)
        } else {
            self.lu_bishop_attacks(other) & other.lu_bishop_attacks(*self)
        }
    }

    /// Return a bitboard of the common axis shared between this single bit
    /// bitboard and another single bit bitboard
    pub fn common_axis(&self, other: BB) -> BB {
        debug_assert_eq!(self.0.count_ones(), 1);
        debug_assert_eq!(other.0.count_ones(), 1);
        debug_assert_ne!(*self, other);
        let (this_sq, other_sq) = (self.to_index(), other.to_index());
        let translation = (this_sq as i32 - other_sq as i32).abs();
        if translation % 9 == 0 {
            // Diagonal translation
            self.lu_diagonal_mask()
        } else if translation % 8 == 0 {
            // Vertical translation
            self.lu_file_mask()
        } else if translation % 7 == 0 && this_sq / 8 != other_sq / 8 {
            // Anti-diagonal translation
            self.lu_anti_diagonal_mask()
        } else if translation < 8 {
            // Horizontal translation
            self.lu_rank_mask()
        } else {
            panic!(
                "Squares {} and {} cannot be connected by a common axis",
                this_sq, other_sq
            )
        }
    }

    /// Convert from algebraic notation e.g. a5 to a one bit bitboard
    pub fn from_algebraic(algebraic: &str) -> Result<BB, uci::RuntimeError> {
        let chars: Vec<char> = algebraic.chars().collect();
        if chars.len() != 2 {
            return Err(uci::RuntimeError::ParseAlgebraicError(
                algebraic.to_string(),
            ));
        }
        if !chars[0].is_alphabetic() || !chars[1].is_numeric() {
            return Err(uci::RuntimeError::ParseAlgebraicError(
                algebraic.to_string(),
            ));
        }
        let file = chars[0].to_ascii_lowercase() as u8 - ascii::LOWER_A as u8;
        let rank = chars[1] as u8 - ascii::ZERO as u8;
        if rank <= 8 && file <= 8 {
            Ok(BB(1 << (file + (rank - 1) * 8)))
        } else {
            Err(uci::RuntimeError::ParseAlgebraicError(
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
        let index = self.ils1b();
        let rank_index = (index / 8) as u8;
        let file_index = (index % 8) as u8;
        let rank = (ascii::ZERO as u8 + rank_index + 1) as char;
        let file = (ascii::LOWER_A as u8 + file_index) as char;
        format!("{}{}", file, rank)
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
        return out;
    }
}

impl std::cmp::PartialEq for BB {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}

impl std::cmp::Eq for BB {}

impl std::ops::BitAnd for BB {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BB(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BB {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::ops::BitOr for BB {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BB(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BB {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl std::ops::BitXor for BB {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BB(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BB {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl std::ops::Mul<u64> for BB {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        BB(self.0.wrapping_mul(rhs))
    }
}

impl std::ops::Not for BB {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::ops::Shl<usize> for BB {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        BB(self.0 << rhs)
    }
}

impl std::ops::Shr<usize> for BB {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        BB(self.0 >> rhs)
    }
}

impl std::ops::Sub for BB {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BB(self.0.wrapping_sub(rhs.0))
    }
}

impl std::ops::SubAssign for BB {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0)
    }
}

impl std::iter::Iterator for BB {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == EMPTY_BB.0 {
            return None;
        }
        let lsb = 1 << self.0.trailing_zeros();
        self.0 ^= lsb;
        return Some(BB(lsb));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_bitmask_to_algebraic() {
        let expected = "f8";
        let input = BB::from_index(61);
        let result = &input.to_algebraic()[..];
        assert_eq!(expected, result)
    }

    #[test]
    fn test_squares_to_bitboard() {
        let squares = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let bitboard = BB::from_indices(squares);
        assert_eq!(bitboard, RANK_1);
    }

    #[test]
    fn test_get_ls1b() {
        let bb = BB::from_indices(vec![19, 30]);
        let lsb = bb.ls1b();
        assert_eq!(BB::from_index(19), lsb);
    }

    #[test]
    fn test_get_ilsb() {
        let bitboard = BB::from_indices(vec![41, 55]);
        let ilsb = bitboard.ils1b();
        assert_eq!(ilsb, 41);
    }

    #[test]
    fn test_forward_scan() {
        let scan_result = FILE_E.forward_scan();
        let expected = vec![
            BB::from_index(4),
            BB::from_index(12),
            BB::from_index(20),
            BB::from_index(28),
            BB::from_index(36),
            BB::from_index(44),
            BB::from_index(52),
            BB::from_index(60),
        ];
        assert_eq!(scan_result, expected);
    }

    #[test_case(Axis::Rank, vec![17, 19, 23, 35], 19, vec![17, 18, 20, 21, 22, 23];"RANK")]
    #[test_case(Axis::File, vec![20, 44, 18], 20, vec![4, 12, 28, 36, 44];"FILE")]
    #[test_case(Axis::Diagonal, vec![27, 54, 18], 27, vec![18, 36, 45, 54];"DIAG")]
    #[test_case(Axis::AntiDiagonal, vec![6, 13, 34, 41, 43], 34, vec![41, 27, 20, 13];"ADIAG")]
    fn test_hyp_quint(axis: Axis, occ: Vec<usize>, slider: usize, expected: Vec<usize>) {
        let occ = BB::from_indices(occ);
        let slider = BB::from_index(slider);
        let result = slider.hyp_quint(occ, axis);
        let expected = BB::from_indices(expected);
        assert_eq!(result, expected);
    }

    #[ignore]
    #[test]
    fn test_print_bb() {
        let king = square::E1;
        let checker = square::E8;
        let out = king.connect_squares(checker).to_string();
        // let conn = king.connect_squares(checker);
        // let out = conn.to_string();
        print!("{}", out)
    }

    #[test]
    fn test_flip_vertical() {
        let bb = BB(0x8040201);
        let result = bb.flip_vertical();
        assert_eq!(result, BB(0x102040800000000))
    }
}
