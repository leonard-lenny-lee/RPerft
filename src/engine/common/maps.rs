/// Contains the Map struct and associated methods to generate the maps. It 
/// should only be instantiated once at program startup and passed by reference
/// to other part of the program. The maps struct contains all the precalculated
/// masks/maps to speed up move generation and other calculation operations at 
/// runtime. TODO implement singleton to allow only one instantiation of a Map
/// struct and only via the new() constructor static method.

use super::*;
use common::{*, bittools as bt};

pub struct Maps {
    pub knight: [u64; 64],
    pub king: [u64; 64],
    pub rank: [u64; 64],
    pub file: [u64; 64],
    pub diag: [u64; 64],
    pub adiag: [u64; 64],
}

impl Maps {
    pub const fn new() -> Maps {
        return Maps {
            knight: Maps::generate_knight_maps(),
            king: Maps::generate_king_maps(),
            rank: Maps::generate_rank_masks(),
            file: Maps::generate_file_masks(),
            diag: Maps::generate_diagonal_masks(),
            adiag: Maps::generate_antidiagonal_masks(),
        }
    }

    /// Get the attack squares of a single knight
    pub fn get_knight_map(&self, bb: u64) -> u64 {
        self.knight[bt::ilsb(bb)]
    }

    /// Get the attack squares of a king
    pub fn get_king_map(&self, bb: u64) -> u64 {
        self.king[bt::ilsb(bb)]
    }

    /// Get the corresponding rank mask of a square
    pub fn get_rank_map(&self, bb: u64) -> u64 {
        self.rank[bt::ilsb(bb)]
    }

    /// Get the corresponding file mask of a square
    pub fn get_file_map(&self, bb: u64) -> u64 {
        self.file[bt::ilsb(bb)]
    }

    /// Get the corresponding diagonal mask of a square
    pub fn get_diag_map(&self, bb: u64) -> u64 {
        self.diag[bt::ilsb(bb)]
    }

    /// Get the corresponding anti-diagonal mask of a square
    pub fn get_adiag_map(&self, bb: u64) -> u64 {
        self.adiag[bt::ilsb(bb)]
    }

    const fn generate_knight_maps() -> [u64; 64] {
        let mut maps: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            let mut map: u64 = 0;
            let origin = 1 << i;
            map |= bt::no_no_ea(origin);
            map |= bt::no_ea_ea(origin);
            map |= bt::so_ea_ea(origin);
            map |= bt::so_so_ea(origin);
            map |= bt::so_so_we(origin);
            map |= bt::so_we_we(origin);
            map |= bt::no_we_we(origin);
            map |= bt::no_no_we(origin);
            maps[i] = map;
            i += 1;
        }
        return maps;   
    }
    
    const fn generate_king_maps() -> [u64; 64] {
        let mut maps: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            let mut map: u64 = 0;
            let origin: u64 = 1 << i;
            map |= bt::north_one(origin);
            map |= bt::nort_east(origin);
            map |= bt::east_one(origin);
            map |= bt::sout_east(origin);
            map |= bt::south_one(origin);
            map |= bt::sout_west(origin);
            map |= bt::west_one(origin);
            map |= bt::nort_west(origin);
            maps[i] = map;
            i += 1;
        }
        return maps;
    }
    
    const fn generate_rank_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            match i / 8 {
                0 => masks[i] = RANK_1,
                1 => masks[i] = RANK_2,
                2 => masks[i] = RANK_3,
                3 => masks[i] = RANK_4,
                4 => masks[i] = RANK_5,
                5 => masks[i] = RANK_6,
                6 => masks[i] = RANK_7,
                7 => masks[i] = RANK_8,
                _ => (),
            }
            i += 1;
        }
        return masks;
    }
    
    const fn generate_file_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            match i % 8 {
                0 => masks[i] = FILE_A,
                1 => masks[i] = FILE_B,
                2 => masks[i] = FILE_C,
                3 => masks[i] = FILE_D,
                4 => masks[i] = FILE_E,
                5 => masks[i] = FILE_F,
                6 => masks[i] = FILE_G,
                7 => masks[i] = FILE_H,
                _ => (),
            }
            i += 1;
        }
        return masks;
    }
    
    const fn generate_diagonal_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            let mut mask: u64 = 1 << i;
            let from_left = i % 8;
            let from_right = 7 - from_left;
            let mut l = 1;
            while l <= from_left {
                let l_trans = i + l * -9;
                if l_trans >= 0 {
                    mask |= 1 << (l_trans);
                } else {
                    break;
                }
                l += 1;
            }
            let mut r = 1;
            while r <= from_right {
                let r_trans = i + r * 9;
                if r_trans < 64 {
                    mask |= 1 << (r_trans);
                } else {
                    break;
                }
                r += 1;
            }
            masks[i as usize] = mask;
            i += 1;
       }
        return masks;
    }
    
    const fn generate_antidiagonal_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        let mut i = 0;
        while i < 64 {
            let mut mask: u64 = 1 << i;
            let from_left = i % 8;
            let from_right = 7 - from_left;
            let mut l = 1;
            while l <= from_left {
                let l_trans = i + l * 7;
                if l_trans < 64 {
                    mask |= 1 << (l_trans);
                } else {
                    break;
                }
                l += 1;
            }
            let mut r = 0;
            while r <= from_right {
                let r_trans = i + r * -7;
                if r_trans >= 0 {
                    mask |= 1 << (r_trans);
                } else {
                    break;
                }
                r += 1;
            }
            masks[i as usize] = mask;
            i += 1;
       }
        return masks;
    }
}