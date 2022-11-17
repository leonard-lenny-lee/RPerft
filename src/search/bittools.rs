/// Contain public functions to carry out bit manipulations

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

pub fn north_one(bb: u64) -> u64 {
    bb << 8
}

pub fn nort_east(bb: u64) -> u64 {
    (bb ^ super::FILE_H) << 9
}

pub fn east_one(bb: u64) -> u64 {
    (bb ^ super::FILE_H) << 1
}

pub fn sout_east(bb: u64) -> u64 {
    (bb ^ super::FILE_H) >> 7
}

pub fn south_one(bb: u64) -> u64 {
    bb >> 8
}

pub fn sout_west(bb: u64) -> u64 {
    (bb ^ super::FILE_A) >> 9
}

pub fn west_one(bb: u64) -> u64 {
    (bb ^ super::FILE_A) >> 1
}

pub fn nort_west(bb: u64) -> u64 {
    (bb ^ super::FILE_A) << 7
}


pub fn no_no_ea(bb: u64) -> u64 {
    (bb ^ super::FILE_H) << 17
}

pub fn no_ea_ea(bb: u64) -> u64 {
    (bb ^ (super::FILE_G | super::FILE_H)) << 10
}

pub fn so_ea_ea(bb: u64) -> u64 {
    (bb ^ (super::FILE_G | super::FILE_H)) >> 6
}

pub fn so_so_ea(bb: u64) -> u64 {
    (bb ^ super::FILE_H) >> 15
}

pub fn so_so_we(bb: u64) -> u64 {
    (bb ^ super::FILE_A) >> 17
}

pub fn so_we_we(bb: u64) -> u64 {
    (bb ^ (super::FILE_A | super::FILE_B)) >> 10
}

pub fn no_we_we(bb: u64) -> u64 {
    (bb ^ (super::FILE_A | super::FILE_B)) << 6
}

pub fn no_no_we(bb: u64) -> u64 {
    (bb ^ super::FILE_A) << 15
}
