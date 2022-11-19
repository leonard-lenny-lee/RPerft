/// Contains commmon functions to carry out bit manipulations

use super::{FILE_A, FILE_B, FILE_G, FILE_H};

pub fn get_lsb(n: &u64) -> u64 {
    1 << n.trailing_zeros()
}

pub fn ilsb(n: &u64) -> usize {
    return n.trailing_zeros() as usize;
}

pub fn forward_scan(mut n: u64) -> Vec<u64> {
    let mut scan_result: Vec<u64> = Vec::new();
    while n != 0 {
        let lsb = get_lsb(&n);
        scan_result.push(lsb);
        n ^= lsb;
    }
    scan_result
}

pub fn hyp_quint(o: u64, s: u64, masks: &[u64; 64]) -> u64 {
    let m = masks[s.trailing_zeros() as usize];
    let mut forward: u64 = o & m;
    let mut reverse: u64 = forward.reverse_bits();
    forward = forward.wrapping_sub(2 * s);
    reverse = reverse.wrapping_sub(2 * s.reverse_bits());
    forward ^= reverse.reverse_bits();
    forward &= m;
    return forward;
}

pub fn create_push_mask(attacker: u64, king: u64) -> u64 {
    assert!(attacker.count_ones() == 1 && king.count_ones() == 1);
    assert!(attacker != king);
    // Calculate direction
    let attacker_sq = attacker.trailing_zeros();
    let king_sq = king.trailing_zeros();
    let push_mask;
    if attacker_sq > king_sq {
        // Attacker must be attacking W, SW, S or SE
        let diff = attacker_sq - king_sq;
        if diff % 9 == 0 {
            push_mask = so_we_ofill(attacker, king)
        } else if diff % 8 == 0 {
            push_mask = sout_ofill(attacker, king)
        } else if diff % 7 == 0 {
            push_mask = so_ea_ofill(attacker, king)
        } else {
            // Assert they are on the same rank
            assert!(attacker_sq / 8 == king_sq / 8);
            push_mask = west_ofill(attacker, king)
        }
    } else {
        // Attacker must be attacking E, NE, N or NW
        let diff = king_sq - attacker_sq;
        if diff % 9 == 0 {
            push_mask = no_ea_ofill(attacker, king)
        } else if diff % 8 == 0 {
            push_mask = nort_ofill(attacker, king)
        } else if diff % 7 == 0 {
            push_mask = no_we_ofill(attacker, king)
        } else {
            assert!(attacker_sq / 8 == king_sq / 8);
            push_mask = east_ofill(attacker, king)
        }
    }
    return push_mask ^ attacker;
}

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

pub fn nort_fill(mut bb: u64) -> u64 {
    bb |= bb << 8;
    bb |= bb << 16;
    bb |= bb << 32;
    return bb;
}

pub fn sout_fill(mut bb: u64) -> u64 {
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    return bb
}

pub fn east_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 << 1);
    let m_3 = m_2 & (m_2 << 2);
    bb |= m_1 & (bb << 1);
    bb |= m_2 & (bb << 2);
    bb |= m_3 & (bb << 4);
    return bb
}

pub fn no_ea_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 << 9);
    let m_3 = m_2 & (m_2 << 18);
    bb |= m_1 & (bb << 9);
    bb |= m_2 & (bb << 18);
    bb |= m_3 & (bb << 36);
    return bb
}

pub fn so_ea_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_A;
    let m_2 = m_1 & (m_1 >> 7);
    let m_3 = m_2 & (m_2 >> 14);
    bb |= m_1 & (bb >> 7);
    bb |= m_2 & (bb >> 14);
    bb |= m_3 & (bb >> 28);
    return bb
}

pub fn west_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 >> 1);
    let m_3 = m_2 & (m_2 >> 2);
    bb |= m_1 & (bb >> 1);
    bb |= m_2 & (bb >> 2);
    bb |= m_3 & (bb >> 4);
    return bb
}

pub fn so_we_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 >> 9);
    let m_3 = m_2 & (m_2 >> 18);
    bb |= m_1 & (bb >> 9);
    bb |= m_2 & (bb >> 18);
    bb |= m_3 & (bb >> 36);
    return bb
}

pub fn no_we_fill(mut bb: u64) -> u64 {
    let m_1 = !FILE_H;
    let m_2 = m_1 & (m_1 << 7);
    let m_3 = m_2 & (m_2 << 14);
    bb |= m_1 & (bb << 7);
    bb |= m_2 & (bb << 14);
    bb |= m_3 & (bb << 28);
    return bb
}

// Occluded fills will only fill bits intervening points on two bitboards
pub fn nort_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_1 |= bb_2 & (bb_1 << 8);
    bb_2 &= bb_2 << 8;
    bb_1 |= bb_2 & (bb_1 << 16);
    bb_2 &= bb_2 << 16;
    bb_1 |= bb_2 & (bb_1 << 32);
    return bb_1
}

pub fn sout_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_1 |= bb_2 & (bb_1 >> 8);
    bb_2 &= bb_2 >> 8;
    bb_1 |= bb_2 & (bb_1 >> 16);
    bb_2 &= bb_2 >> 16;
    bb_1 |= bb_2 & (bb_1 >> 32);
    return bb_1
}

pub fn east_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_A;
    bb_1 |= bb_2 & (bb_1 << 1);
    bb_2 &= bb_2 << 1;
    bb_1 |= bb_2 & (bb_1 << 2);
    bb_2 &= bb_2 << 2;
    bb_1 |= bb_2 & (bb_1 << 4);
    return bb_1
}

pub fn west_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_H;
    bb_1 |= bb_2 & (bb_1 >> 1);
    bb_2 &= bb_2 >> 1;
    bb_1 |= bb_2 & (bb_1 >> 2);
    bb_2 &= bb_2 >> 2;
    bb_1 |= bb_2 & (bb_1 >> 4);
    return bb_1
}

pub fn no_ea_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_A;
    bb_1 |= bb_2 & (bb_1 << 9);
    bb_2 &= bb_2 << 9;
    bb_1 |= bb_2 & (bb_1 << 18);
    bb_2 &= bb_2 << 18;
    bb_1 |= bb_2 & (bb_1 << 36);
    return bb_1
}

pub fn so_ea_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_A;
    bb_1 |= bb_2 & (bb_1 >> 7);
    bb_2 &= bb_2 >> 7;
    bb_1 |= bb_2 & (bb_1 >> 14);
    bb_2 &= bb_2 >> 14;
    bb_1 |= bb_2 & (bb_1 >> 28);
    return bb_1
}

pub fn no_we_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_H;
    bb_1 |= bb_2 & (bb_1 << 7);
    bb_2 &= bb_2 << 7;
    bb_1 |= bb_2 & (bb_1 << 14);
    bb_2 &= bb_2 << 14;
    bb_1 |= bb_2 & (bb_1 << 28);
    return bb_1
}

pub fn so_we_ofill(mut bb_1: u64, mut bb_2: u64) -> u64 {
    bb_2 = !bb_2;
    bb_2 ^= FILE_H;
    bb_1 |= bb_2 & (bb_1 >> 9);
    bb_2 &= bb_2 >> 9;
    bb_1 |= bb_2 & (bb_1 >> 18);
    bb_2 &= bb_2 >> 18;
    bb_1 |= bb_2 & (bb_1 >> 36);
    return bb_1
}

pub fn north_one(bb: u64) -> u64 {
    bb << 8
}

pub fn nort_east(bb: u64) -> u64 {
    (bb ^ FILE_H) << 9
}

pub fn east_one(bb: u64) -> u64 {
    (bb ^ FILE_H) << 1
}

pub fn sout_east(bb: u64) -> u64 {
    (bb ^ FILE_H) >> 7
}

pub fn south_one(bb: u64) -> u64 {
    bb >> 8
}

pub fn sout_west(bb: u64) -> u64 {
    (bb ^ FILE_A) >> 9
}

pub fn west_one(bb: u64) -> u64 {
    (bb ^ FILE_A) >> 1
}

pub fn nort_west(bb: u64) -> u64 {
    (bb ^ FILE_A) << 7
}


pub fn no_no_ea(bb: u64) -> u64 {
    (bb ^ FILE_H) << 17
}

pub fn no_ea_ea(bb: u64) -> u64 {
    (bb ^ (FILE_G | FILE_H)) << 10
}

pub fn so_ea_ea(bb: u64) -> u64 {
    (bb ^ (FILE_G | FILE_H)) >> 6
}

pub fn so_so_ea(bb: u64) -> u64 {
    (bb ^ FILE_H) >> 15
}

pub fn so_so_we(bb: u64) -> u64 {
    (bb ^ FILE_A) >> 17
}

pub fn so_we_we(bb: u64) -> u64 {
    (bb ^ (FILE_A | FILE_B)) >> 10
}

pub fn no_we_we(bb: u64) -> u64 {
    (bb ^ (FILE_A | FILE_B)) << 6
}

pub fn no_no_we(bb: u64) -> u64 {
    (bb ^ FILE_A) << 15
}
