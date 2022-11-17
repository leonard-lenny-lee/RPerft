/// Contains the Map struct and associated methods to generate the maps.
// TODO implement singleton to allow one instantiation of a Map struct, using
// only the new() constructor.

use std::collections::HashMap;

pub struct Maps {
    pub knight: [u64; 64],
    pub dknight: HashMap<u64, u64>,
    pub king: [u64; 64],
    pub rank: [u64; 64],
    pub file: [u64; 64],
    pub diag: [u64; 64],
    pub adiag: [u64; 64],
}

impl Maps {
    pub fn new() -> Maps {
        return Maps {
            knight: Maps::generate_knight_maps(),
            dknight: Maps::generate_dbl_knight_maps(),
            king: Maps::generate_king_maps(),
            rank: Maps::generate_rank_masks(),
            file: Maps::generate_file_masks(),
            diag: Maps::generate_diagonal_masks(),
            adiag: Maps::generate_antidiagonal_masks(),
        }
    }

    fn generate_knight_maps() -> [u64; 64] {
        let mut maps: [u64; 64] = [0; 64];
        for i in 0..64 {
            let mut map: u64 = 0;
            let origin = 1 << i;
            map |= super::bittools::no_no_ea(origin);
            map |= super::bittools::no_ea_ea(origin);
            map |= super::bittools::so_ea_ea(origin);
            map |= super::bittools::so_so_ea(origin);
            map |= super::bittools::so_so_we(origin);
            map |= super::bittools::so_we_we(origin);
            map |= super::bittools::no_we_we(origin);
            map |= super::bittools::no_no_we(origin);
            maps[i] = map;
        }
        return maps;   
    }

    fn generate_dbl_knight_maps() -> HashMap<u64, u64> {
        let mut maps: HashMap<u64, u64> = HashMap::new();
        for i in 0..64 {
            for j in 0..64 {
                let mut map: u64 = 0;
                let origin = 1 << i | 1 << j;
                map |= super::bittools::no_no_ea(origin);
                map |= super::bittools::no_ea_ea(origin);
                map |= super::bittools::so_ea_ea(origin);
                map |= super::bittools::so_so_ea(origin);
                map |= super::bittools::so_so_we(origin);
                map |= super::bittools::so_we_we(origin);
                map |= super::bittools::no_we_we(origin);
                map |= super::bittools::no_no_we(origin);
                maps.insert(origin, map);
            }
        }
        return maps;
    }
    
    fn generate_king_maps() -> [u64; 64] {
        let mut maps: [u64; 64] = [0; 64];
        for i in 0..64 {
            let mut map: u64 = 0;
            let origin: u64 = 1 << i;
            map |= super::bittools::north_one(origin);
            map |= super::bittools::nort_east(origin);
            map |= super::bittools::east_one(origin);
            map |= super::bittools::sout_east(origin);
            map |= super::bittools::south_one(origin);
            map |= super::bittools::sout_west(origin);
            map |= super::bittools::west_one(origin);
            map |= super::bittools::nort_west(origin);
            maps[i] = map;
        }
        return maps;
    }
    
    fn generate_rank_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        for i in 0..64 {
            match i / 8 {
                0 => masks[i] = super::RANK_1,
                1 => masks[i] = super::RANK_2,
                2 => masks[i] = super::RANK_3,
                3 => masks[i] = super::RANK_4,
                4 => masks[i] = super::RANK_5,
                5 => masks[i] = super::RANK_6,
                6 => masks[i] = super::RANK_7,
                7 => masks[i] = super::RANK_8,
                _ => (),
            }
        }
        return masks;
    }
    
    fn generate_file_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        for i in 0..64 {
            match i % 8 {
                0 => masks[i] = super::FILE_A,
                1 => masks[i] = super::FILE_B,
                2 => masks[i] = super::FILE_C,
                3 => masks[i] = super::FILE_D,
                4 => masks[i] = super::FILE_E,
                5 => masks[i] = super::FILE_F,
                6 => masks[i] = super::FILE_G,
                7 => masks[i] = super::FILE_H,
                _ => (),
            }
        }
        return masks;
    }
    
    fn generate_diagonal_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        for i in 0..64 {
            let mut mask: u64 = 1 << i;
            let from_left = i % 8;
            let from_right = 7 - from_left;
            for l in 1..from_left+1 {
                let l_trans = i + l * -9;
                if l_trans >= 0 {
                    mask |= 1 << (l_trans);
                } else {
                    break;
                }
            }
            for r in 1..from_right+1 {
                let r_trans = i + r * 9;
                if r_trans < 64 {
                    mask |= 1 << (r_trans);
                } else {
                    break;
                }
            }
            masks[i as usize] = mask;
       }
        return masks;
    }
    
    fn generate_antidiagonal_masks() -> [u64; 64] {
        let mut masks: [u64; 64] = [0; 64];
        for i in 0..64 {
            let mut mask: u64 = 1 << i;
            let from_left = i % 8;
            let from_right = 7 - from_left;
            for l in 1..from_left+1 {
                let l_trans = i + l * 7;
                if l_trans < 64 {
                    mask |= 1 << (l_trans);
                } else {
                    break;
                }
            }
            for r in 1..from_right+1 {
                let r_trans = i + r * -7;
                if r_trans >= 0 {
                    mask |= 1 << (r_trans);
                } else {
                    break;
                }
            }
            masks[i as usize] = mask;
       }
        return masks;
    }
}