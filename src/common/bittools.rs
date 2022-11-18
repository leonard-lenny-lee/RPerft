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

pub fn create_ray_mask(n_1: u64, n_2: u64) {
    // TODO write ray mask generation function
    assert!(n_1.count_ones() == 1 && n_2.count_ones() == 1);
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
