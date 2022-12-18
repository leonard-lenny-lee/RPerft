/// Toolkit to carry out bit manipulations

use super::*;
use position::PieceSet;

/// Convert algebraic notation of a square e.g. a5 to a single bit mask at
/// the corresponding index
pub fn algebraic_to_bitmask(algebraic: &str) -> u64 {
    assert!(algebraic.len() == 2);
    let chars: Vec<char> = algebraic.chars().collect();
    assert!(chars[0].is_alphabetic());
    assert!(chars[1].is_numeric());
    let file = chars[0] as u8 - ASCIIBases::LowerA as u8;
    let rank = chars[1] as u8 - ASCIIBases::Zero as u8;
    assert!(file <= 8);
    assert!(file <= 8);
    1 << (file + (rank - 1) * 8)
}

/// Convert a single bit mask to algebraic notation
pub fn bitmask_to_algebraic(bb: u64) -> String {
    let index = ilsb(bb);
    let rank_index = (index / 8) as u8;
    let file_index = (index % 8) as u8;
    let rank = (ASCIIBases::Zero as u8 + rank_index + 1) as char;
    let file = (ASCIIBases::LowerA as u8 + file_index) as char;
    format!("{}{}", file, rank)
}

/// Convert a bitboard representation (u64) into a string representation
pub fn bitboard_to_string(n: u64) -> String {
    let mut out = String::new();
    out.push_str("   --- --- --- --- --- --- --- --- \n8 ");
    for i in 0..64 {
        if i % 8 == 0 && i != 0 {
            let rank = &(8 - (i / 8)).to_string()[..];
            out.push_str("|\n   --- --- --- --- --- --- --- --- \n");
            out.push_str(rank);
            out.push(' ')
        }
        if ((1 << (7 - i / 8) * 8 + (i % 8)) & n) != 0 {
            out.push_str("| x ")
        } else {
            out.push_str("|   ")
        }
    }
    out.push_str(
        "|\n   --- --- --- --- --- --- --- --- \n    a   b   c   d   e   f   g   h ");
    return out;
}

/// Convert position (white and black pieceset) into a string representation
pub fn piecesets_to_string(w_pieces: PieceSet, b_pieces: PieceSet) -> String {
    let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
    let w_array = w_pieces.as_array();
    let b_array = b_pieces.as_array();
    for i in 1..7 {
        let mut char;
        match i {
            1 => char = 'p',
            2 => char = 'r',
            3 => char = 'n',
            4 => char = 'b',
            5 => char = 'q',
            6 => char = 'k',
            _ => char = ' ',
        };
        for bit in forward_scan(b_array[i]) {
            let index = ilsb(bit);
            let x = index / 8;
            let y = index % 8;
            array[x][y] = char;
        }
        char.make_ascii_uppercase();
        for bit in forward_scan(w_array[i]) {
            let index = ilsb(bit);
            let x = index / 8;
            let y = index % 8;
            array[x][y] = char;
        }
    };
    let mut out = String::new();
    out.push_str("   --- --- --- --- --- --- --- --- \n8 ");
    for i in 0..8 {
        let i2 = 7 - i;
        let row = array[i2];
        if i != 0 {
            let rank = &(8 - i).to_string()[..];
            out.push_str("|\n   --- --- --- --- --- --- --- --- \n");
            out.push_str(rank);
            out.push(' ');
        }
        for c in row {
            out.push_str("| ");
            out.push(c);
            out.push(' ')
        }
    }
    out.push_str(
        "|\n   --- --- --- --- --- --- --- --- \n    a   b   c   d   e   f   g   h "
    );
    return out
}

/// Accepts a vector of bitboard indices and returns a bitboard with the
/// corresponding index positions set to 1
pub fn squares_to_bitboard(squares: Vec<i32>) -> u64 {
    let mut out: u64 = EMPTY_BB;
    for square in squares {
        out |= 1 << square;
    }
    return out;
}

/// Returns the isolated least significant bit
pub fn get_lsb(n: u64) -> u64 {
    1 << n.trailing_zeros()
}

/// Pops off and returns the least significant set bit
pub fn pop_lsb(n: &mut u64) -> u64 {
    let lsb: u64 = 1 << n.trailing_zeros();
    *n ^= lsb;
    return lsb
}

/// Returns the index of the least significant bit as a usize
pub fn ilsb(n: u64) -> usize {
    return n.trailing_zeros() as usize;
}

/// Pops off and returns the least significant set bit index
pub fn pop_ilsb(n: &mut u64) -> usize {
    let ilsb = n.trailing_zeros() as usize;
    *n ^= 1 << ilsb;
    return ilsb
}

/// Return the index of the least significant bit as a u8
pub fn ilsb_u8(n: u64) -> u8 {
    return n.trailing_zeros() as u8
}

/// Decomposes a bitboard into a vector of single bit boardboard
pub fn forward_scan(mut n: u64) -> Vec<u64> {
    let mut scan_result: Vec<u64> = Vec::new();
    while n != 0 {
        let lsb = get_lsb(n);
        scan_result.push(lsb);
        n ^= lsb;
    }
    scan_result
}

/// Uses the o-2s bit fiddling technique to find valid squares for sliding
/// pieces, taking into account the occupancy of the current board
pub const fn hyp_quint(o: u64, s: u64, masks: &[u64; 64]) -> u64 {
    let m = masks[s.trailing_zeros() as usize];
    let mut forward: u64 = o & m;
    let mut reverse: u64 = forward.reverse_bits();
    forward = forward.wrapping_sub(s.wrapping_mul(2));
    reverse = reverse.wrapping_sub(s.reverse_bits().wrapping_mul(2));
    forward ^= reverse.reverse_bits();
    forward &= m;
    return forward;
}

/// Combined hyperbola quintesscence for the horizontal and vertical directions
/// - the rook movement pattern
pub const fn hv_hyp_quint(o: u64, s: u64) -> u64 {
    let mut result = EMPTY_BB;
    result |= hyp_quint(o, s, &MAPS.file);
    result |= hyp_quint(o, s, &MAPS.rank);
    return result;
}

/// Combined hyperbola quintessence for the diagonal and anti-diagonal
/// directions - the bishop movement pattern
pub const fn da_hyp_quint(o: u64, s: u64) -> u64 {
    let mut result = EMPTY_BB;
    result |= hyp_quint(o, s, &MAPS.diag);
    result |= hyp_quint(o, s, &MAPS.adiag);
    return result;
}

/// Combined hyperbola quintessence for all four directions - the queen
/// movement pattern
pub fn all_hyp_quint(o: u64, s: u64) -> u64 {
    let mut result = EMPTY_BB;
    result |= hyp_quint(o, s, &MAPS.file);
    result |= hyp_quint(o, s, &MAPS.rank);
    result |= hyp_quint(o, s, &MAPS.diag);
    result |= hyp_quint(o, s, &MAPS.adiag);
    return result;
}

/// Given two squares, calculates the appropriate direction and fills the
/// intervening squares using the appropriate KS algorithm. Panics if they
/// cannot be filled in the 8 possible directions.
pub fn connect_squares(square_one: u64, square_two: u64) -> u64 {
    assert!(square_one.count_ones() == 1 && square_two.count_ones() == 1);
    assert!(square_one != square_two);
    // Calculate direction
    let sq_one_index = square_one.trailing_zeros();
    let sq_two_index = square_two.trailing_zeros();
    let push_mask;
    if sq_one_index > sq_two_index {
        // Attacker must be attacking W, SW, S or SE
        let diff = sq_one_index - sq_two_index;
        if diff < 8 && sq_one_index / 8 == sq_two_index / 8 {
            // They are on the same rank
            push_mask = west_ofill(square_one, square_two)
        } else if diff % 9 == 0 {
            push_mask = so_we_ofill(square_one, square_two)
        } else if diff % 8 == 0 {
            push_mask = sout_ofill(square_one, square_two)
        } else if diff % 7 == 0 {
            push_mask = so_ea_ofill(square_one, square_two)
        } else {
            panic!("Squares cannot be connected")
        }
    } else {
        // Attacker must be attacking E, NE, N or NW
        let diff = sq_two_index - sq_one_index;
        if diff < 8 && sq_one_index / 8 == sq_two_index / 8 {
            push_mask = east_ofill(square_one, square_two)
        } else if diff % 9 == 0 {
            push_mask = no_ea_ofill(square_one, square_two)
        } else if diff % 8 == 0 {
            push_mask = nort_ofill(square_one, square_two)
        } else if diff % 7 == 0 {
            push_mask = no_we_ofill(square_one, square_two)
        } else {
            panic!("Squares cannot be connected")
        }
    }
    return push_mask ^ square_one;
}

/// Give the origin square, it calculates the appropriate firection to fill 
/// towards the piece square and fills in that direction until the edge of the
/// board. Panics if they cannot be filled in the 8 possible directions
pub fn ray_axis(origin: u64, piece: u64) -> u64 {
    assert!(origin.count_ones() == 1 && piece.count_ones() == 1);
    assert!(origin != piece);
    // Calculate direction
    let origin_sq = origin.trailing_zeros();
    let piece_sq = piece.trailing_zeros();
    let ray;
    if origin_sq > piece_sq {
        // must be pointing W, SW, S or SE
        let diff = origin_sq - piece_sq;
        if diff % 9 == 0 {
            ray = so_we_fill(origin)
        } else if diff % 8 == 0 {
            ray = sout_fill(origin)
        } else if diff % 7 == 0 {
            ray = so_ea_fill(origin)
        } else {
            // Assert they are on the same rank
            assert!(origin_sq / 8 == piece_sq / 8);
            ray = west_fill(origin)
        }
    } else {
        // Attacker must be attacking E, NE, N or NW
        let diff = piece_sq - origin_sq;
        if diff % 9 == 0 {
            ray = no_ea_fill(origin)
        } else if diff % 8 == 0 {
            ray = nort_fill(origin)
        } else if diff % 7 == 0 {
            ray = no_we_fill(origin)
        } else {
            assert!(origin_sq / 8 == piece_sq / 8);
            ray = east_fill(origin)
        }
    }
    return ray ^ origin;
}

/// Fills all squares north of any bits
pub fn nort_fill(mut bb: u64) -> u64 {
    bb |= bb << 8;
    bb |= bb << 16;
    bb |= bb << 32;
    return bb;
}

/// Fills all squares south of any bits
pub fn sout_fill(mut bb: u64) -> u64 {
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    return bb
}

/// Fills all squares east of any bits
pub fn east_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 << 1);
    let m_3 = m_2 & (m_2 << 2);
    bb |= m_1 & (bb << 1);
    bb |= m_2 & (bb << 2);
    bb |= m_3 & (bb << 4);
    return bb
}

/// Fills all squares north east of any bits
pub fn no_ea_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 << 9);
    let m_3 = m_2 & (m_2 << 18);
    bb |= m_1 & (bb << 9);
    bb |= m_2 & (bb << 18);
    bb |= m_3 & (bb << 36);
    return bb
}

/// Fills all squares south east of any bits
pub fn so_ea_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 >> 7);
    let m_3 = m_2 & (m_2 >> 14);
    bb |= m_1 & (bb >> 7);
    bb |= m_2 & (bb >> 14);
    bb |= m_3 & (bb >> 28);
    return bb
}

/// Fills all squares west of any bits
pub fn west_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 >> 1);
    let m_3 = m_2 & (m_2 >> 2);
    bb |= m_1 & (bb >> 1);
    bb |= m_2 & (bb >> 2);
    bb |= m_3 & (bb >> 4);
    return bb
}

/// Fills all squares south west of any bits
pub fn so_we_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 >> 9);
    let m_3 = m_2 & (m_2 >> 18);
    bb |= m_1 & (bb >> 9);
    bb |= m_2 & (bb >> 18);
    bb |= m_3 & (bb >> 36);
    return bb
}

/// Fills all squares north west of any bits
pub fn no_we_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 << 7);
    let m_3 = m_2 & (m_2 << 14);
    bb |= m_1 & (bb << 7);
    bb |= m_2 & (bb << 14);
    bb |= m_3 & (bb << 28);
    return bb
}

/// Fills all squares north of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn nort_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_1 |= bb_2 & (bb_1 << 8);
    bb_2 &= bb_2 << 8;
    bb_1 |= bb_2 & (bb_1 << 16);
    bb_2 &= bb_2 << 16;
    bb_1 |= bb_2 & (bb_1 << 32);
    return bb_1
}

/// Fills all squares south of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn sout_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_1 |= bb_2 & (bb_1 >> 8);
    bb_2 &= bb_2 >> 8;
    bb_1 |= bb_2 & (bb_1 >> 16);
    bb_2 &= bb_2 >> 16;
    bb_1 |= bb_2 & (bb_1 >> 32);
    return bb_1
}

/// Fills all squares east of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn east_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_A;
    bb_1 |= bb_2 & (bb_1 << 1);
    bb_2 &= bb_2 << 1;
    bb_1 |= bb_2 & (bb_1 << 2);
    bb_2 &= bb_2 << 2;
    bb_1 |= bb_2 & (bb_1 << 4);
    return bb_1
}

/// Fills all squares west of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn west_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_H;
    bb_1 |= bb_2 & (bb_1 >> 1);
    bb_2 &= bb_2 >> 1;
    bb_1 |= bb_2 & (bb_1 >> 2);
    bb_2 &= bb_2 >> 2;
    bb_1 |= bb_2 & (bb_1 >> 4);
    return bb_1
}

/// Fills all squares north east of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn no_ea_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_A;
    bb_1 |= bb_2 & (bb_1 << 9);
    bb_2 &= bb_2 << 9;
    bb_1 |= bb_2 & (bb_1 << 18);
    bb_2 &= bb_2 << 18;
    bb_1 |= bb_2 & (bb_1 << 36);
    return bb_1
}

/// Fills all squares south east of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn so_ea_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_A;
    bb_1 |= bb_2 & (bb_1 >> 7);
    bb_2 &= bb_2 >> 7;
    bb_1 |= bb_2 & (bb_1 >> 14);
    bb_2 &= bb_2 >> 14;
    bb_1 |= bb_2 & (bb_1 >> 28);
    return bb_1
}

/// Fills all squares north west of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn no_we_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_H;
    bb_1 |= bb_2 & (bb_1 << 7);
    bb_2 &= bb_2 << 7;
    bb_1 |= bb_2 & (bb_1 << 14);
    bb_2 &= bb_2 << 14;
    bb_1 |= bb_2 & (bb_1 << 28);
    return bb_1
}

/// Fills all squares south west of any bits in bitboard one until it hits any
/// bits in bitboard two
pub fn so_we_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 &= !FILE_H;
    bb_1 |= bb_2 & (bb_1 >> 9);
    bb_2 &= bb_2 >> 9;
    bb_1 |= bb_2 & (bb_1 >> 18);
    bb_2 &= bb_2 >> 18;
    bb_1 |= bb_2 & (bb_1 >> 36);
    return bb_1
}

pub fn nort_attacks(bb_1: u64, bb_2: u64) -> u64 {
    north_one(nort_ofill(bb_1, bb_2))
}

pub fn sout_attacks(bb_1: u64, bb_2: u64) -> u64 {
    south_one(sout_ofill(bb_1, bb_2))
}

pub fn east_attacks(bb_1: u64, bb_2: u64) -> u64 {
    east_one(east_ofill(bb_1, bb_2))
}

pub fn west_attacks(bb_1: u64, bb_2: u64) -> u64 {
    west_one(west_ofill(bb_1, bb_2))
}

pub fn no_ea_attacks(bb_1: u64, bb_2: u64) -> u64 {
    nort_east(no_ea_ofill(bb_1, bb_2))
}

pub fn no_we_attacks(bb_1: u64, bb_2: u64) -> u64 {
    nort_west(no_we_ofill(bb_1, bb_2))
}

pub fn so_ea_attacks(bb_1: u64, bb_2: u64) -> u64 {
    sout_east(so_ea_ofill(bb_1, bb_2))
}

pub fn so_we_attacks(bb_1: u64, bb_2: u64) -> u64 {
    sout_west(so_we_ofill(bb_1, bb_2))
}

pub fn file_attacks(bb_1: u64, bb_2: u64) -> u64 {
    nort_attacks(bb_1, bb_2) | sout_attacks(bb_1, bb_2)
}

pub fn rank_attacks(bb_1: u64, bb_2: u64) -> u64 {
    east_attacks(bb_1, bb_2) | west_attacks(bb_1, bb_2)
}

pub fn diag_attacks(bb_1: u64, bb_2: u64) -> u64 {
    no_ea_attacks(bb_1, bb_2) | so_we_attacks(bb_1, bb_2)
}

pub fn adiag_attacks(bb_1: u64, bb_2: u64) -> u64 {
    no_we_attacks(bb_1, bb_2) | so_ea_attacks(bb_1, bb_2)
}

pub fn rook_attacks(bb_1: u64, bb_2: u64) -> u64 {
    nort_attacks(bb_1, bb_2) 
    | sout_attacks(bb_1, bb_2)
    | west_attacks(bb_1, bb_2) 
    | east_attacks(bb_1, bb_2)
}

pub fn bishop_attacks(bb_1: u64, bb_2: u64) -> u64 {
    no_ea_attacks(bb_1, bb_2)
    | so_ea_attacks(bb_1, bb_2)
    | no_we_attacks(bb_1, bb_2)
    | so_we_attacks(bb_1, bb_2)
}

pub const fn north_one(bb: u64) -> u64 {
    bb << 8
}

pub const fn north_two(bb: u64) -> u64 {
    bb << 16
}

pub const fn nort_east(bb: u64) -> u64 {
    (bb & !FILE_H) << 9
}

pub const fn east_one(bb: u64) -> u64 {
    (bb & !FILE_H) << 1
}

pub const fn east_two(bb: u64) -> u64 {
    (bb & !(FILE_G | FILE_H)) << 2
}

pub const fn sout_east(bb: u64) -> u64 {
    (bb & !FILE_H) >> 7
}

pub const fn south_one(bb: u64) -> u64 {
    bb >> 8
}

pub const fn south_two(bb: u64) -> u64 {
    bb >> 16
}

pub const fn sout_west(bb: u64) -> u64 {
    (bb & !FILE_A) >> 9
}

pub const fn west_one(bb: u64) -> u64 {
    (bb & !FILE_A) >> 1
}

pub const fn west_two(bb: u64) -> u64 {
    (bb & !(FILE_A | FILE_B)) >> 2
}

pub const fn nort_west(bb: u64) -> u64 {
    (bb & !FILE_A) << 7
}

pub const fn no_no_ea(bb: u64) -> u64 {
    (bb & !FILE_H) << 17
}

pub const fn no_ea_ea(bb: u64) -> u64 {
    (bb & !(FILE_G | FILE_H)) << 10
}

pub const fn so_ea_ea(bb: u64) -> u64 {
    (bb & !(FILE_G | FILE_H)) >> 6
}

pub const fn so_so_ea(bb: u64) -> u64 {
    (bb & !FILE_H) >> 15
}

pub const fn so_so_we(bb: u64) -> u64 {
    (bb & !FILE_A) >> 17
}

pub const fn so_we_we(bb: u64) -> u64 {
    (bb & !(FILE_A | FILE_B)) >> 10
}

pub const fn no_we_we(bb: u64) -> u64 {
    (bb & !(FILE_A | FILE_B)) << 6
}

pub const fn no_no_we(bb: u64) -> u64 {
    (bb & !FILE_A) << 15
}

pub fn flip_vertical(bb: u64) -> u64 {
    bb.swap_bytes()
}

#[cfg(test)]
mod tests;