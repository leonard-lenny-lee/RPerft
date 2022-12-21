/// Contains the Map struct and associated methods to generate the maps. It 
/// should only be instantiated once at program startup and passed by reference
/// to other part of the program. The maps struct contains all the precalculated
/// masks/maps to speed up move generation and other calculation operations at 
/// runtime. TODO implement singleton to allow only one instantiation of a Map
/// struct and only via the new() constructor static method.

use super::*;

pub struct Maps {
    pub knight_attack_squares: [BB; 64],
    pub king_attack_squares: [BB; 64],
    pub rank_masks: [BB; 64],
    pub file_masks: [BB; 64],
    pub diag_masks: [BB; 64],
    pub adiag_masks: [BB; 64],
}

impl Maps {
    pub const fn new() -> Maps {
        return Maps {
            knight_attack_squares: Maps::generate_knight_attack_maps(),
            king_attack_squares: Maps::generate_king_attack_maps(),
            rank_masks: Maps::generate_rank_masks(),
            file_masks: Maps::generate_file_masks(),
            diag_masks: Maps::generate_diagonal_masks(),
            adiag_masks: Maps::generate_antidiagonal_masks(),
        }
    }

    /// Get the attack squares of a single knight
    pub fn get_knight_map(&self, bb: BB) -> BB {
        self.knight_attack_squares[bb.ils1b()]
    }

    /// Get the attack squares of a king
    pub fn get_king_map(&self, bb: BB) -> BB {
        self.king_attack_squares[bb.ils1b()]
    }

    /// Get the corresponding rank mask of a square
    pub fn get_rank_map(&self, bb: BB) -> BB {
        self.rank_masks[bb.ils1b()]
    }

    /// Get the corresponding file mask of a square
    pub fn get_file_map(&self, bb: BB) -> BB {
        self.file_masks[bb.ils1b()]
    }

    /// Get the corresponding diagonal mask of a square
    pub fn get_diag_map(&self, bb: BB) -> BB {
        self.diag_masks[bb.ils1b()]
    }

    /// Get the corresponding anti-diagonal mask of a square
    pub fn get_adiag_map(&self, bb: BB) -> BB {
        self.adiag_masks[bb.ils1b()]
    }

    const fn generate_knight_attack_maps() -> [BB; 64] {
        let mut maps: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            maps[i] = BB(1 << i).knight_attack_squares();
            i += 1;
        }
        return maps;   
    }
    
    const fn generate_king_attack_maps() -> [BB; 64] {
        let mut maps: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            maps[i] = BB(1 << i).king_attack_squares();
            i += 1;
        }
        return maps;
    }
    
    const fn generate_rank_masks() -> [BB; 64] {
        let mut masks: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            masks[i] = RANK_MASKS[i / 8];
            i += 1;
        }
        return masks;
    }
    
    const fn generate_file_masks() -> [BB; 64] {
        let mut masks: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            masks[i] = FILE_MASKS[i % 8];
            i += 1;
        }
        return masks;
    }
    
    const fn generate_diagonal_masks() -> [BB; 64] {
        let mut masks: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            let origin = BB(1 << i);
            masks[i] = BB(origin.no_ea_fill().0 | origin.so_we_fill().0);
            i += 1;
       }
        return masks;
    }
    
    const fn generate_antidiagonal_masks() -> [BB; 64] {
        let mut masks: [BB; 64] = [BB(0); 64];
        let mut i = 0;
        while i < 64 {
            let origin = BB(1 << i);
            masks[i] = BB(origin.no_we_fill().0 | origin.so_ea_fill().0);
            i += 1
       }
        return masks;
    }
}