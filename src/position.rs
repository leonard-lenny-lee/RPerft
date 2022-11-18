/// Contains the Position struct, which holds the all the bitboards and data
/// to describe the current position, as well as methods to derive other
/// bitboards required for move generation and evaluation

use super::common::*;
use super::search::move_generation::Move;
use super::global::maps::Maps;

pub struct Position {
    pub w_pieces: [u64; 7],
    pub b_pieces: [u64; 7],
    pub occ: u64,
    pub free: u64,
    pub white_to_move: bool,
    pub w_kingside_castle: bool,
    pub b_kingside_castle: bool,
    pub w_queenside_castle: bool,
    pub b_queenside_castle: bool,
    pub en_passant_target_sq: u64,
    pub halfmove_clock: i8,
    pub fullmove_clock: i8,
}

impl Position {

    pub fn new_from_fen(fen: String) -> Position {
        let split_fen: Vec<&str> = fen.split(" ").collect();
        assert!(split_fen.len() == 6);
        let board = split_fen[0];
        // Initialise bitboard
        let mut w_pieces: [u64; 7] = [0; 7];
        let mut b_pieces: [u64; 7] = [0; 7];
        // FILL BITBOARD ROUTINE
        // Split the FEN string at "/"
        let mut split_board: Vec<&str> = board.split("/").collect();
        assert!(split_board.len() == 8);
        // Reverse vector so that 0 index is now at square A1
        split_board.reverse();
        let rev_board = &split_board.join("")[..];
        let mut i = 0;
        for char in rev_board.chars() {
            let mask: u64 = 1 << i;
            if char.is_alphabetic() {
                // If the character is alphabetic, then it represents a piece;
                // populate the relevant bitboard
                match char {
                    'P' => w_pieces[Piece::Pawn as usize] |= mask,
                    'p' => b_pieces[Piece::Pawn as usize] |= mask,
                    'R' => w_pieces[Piece::Rook as usize] |= mask,
                    'r' => b_pieces[Piece::Rook as usize] |= mask,
                    'N' => w_pieces[Piece::Knight as usize] |= mask,
                    'n' => b_pieces[Piece::Knight as usize] |= mask,
                    'B' => w_pieces[Piece::Bishop as usize] |= mask,
                    'b' => b_pieces[Piece::Bishop as usize] |= mask,
                    'Q' => w_pieces[Piece::Queen as usize] |= mask,
                    'q' => b_pieces[Piece::Queen as usize] |= mask,
                    'K' => w_pieces[Piece::King as usize] |= mask,
                    'k' => b_pieces[Piece::King as usize] |= mask,
                    _ => panic!("Invalid character {} found in FEN", char),
                }
                i += 1;
            } else {
                assert!(char.is_numeric());
                // Character represents empty squares so skip over the matching
                // number of index positions.
                i += char.to_digit(10).unwrap();
            }
        }
        assert!(i == 63);
        let occ = w_pieces[0] | b_pieces[0];
        let free = !occ;
        // Populate other fields
        let white_to_move: bool = split_fen[1] == "w";
        let mut w_kingside_castle: bool = false;
        let mut b_kingside_castle: bool = false;
        let mut w_queenside_castle: bool = false;
        let mut b_queenside_castle: bool = false;
        for c in split_fen[2].chars() {
            match c {
                'K' => w_kingside_castle = true,
                'Q' => w_queenside_castle = true,
                'k' => b_kingside_castle = true,
                'q' => b_queenside_castle = true,
                _ => (),
            }
        };
        // Calculate en passant target square
        let mut en_passant_target_sq: u64 = 0;
        let epts: Vec<char> = split_fen[3].chars().collect();
        if epts[0] != '-' {
            assert!(epts.len() == 2);
            let file = epts[0] as u8;
            let rank = epts[1] as u8;
            en_passant_target_sq = 1 << ((file - ASCIIBases::LowerA as u8)
                + (rank - ASCIIBases::Zero as u8) * 8);
        }
        // Calculate clocks
        let halfmove_clock: i8 = split_fen[4].parse().unwrap();
        let fullmove_clock: i8 = split_fen[5].parse().unwrap();
        // Construct struct with calculated values
        return Position {
            w_pieces, b_pieces, occ, free, white_to_move, w_kingside_castle,
            b_kingside_castle, w_queenside_castle, b_queenside_castle,
            en_passant_target_sq, halfmove_clock, fullmove_clock
        }
    }
}

/// Methods to generate the target maps for pawn moves
impl Position {

    pub fn get_wpawn_sgl_pushes(&self) -> u64 {
        self.w_pieces[1] << 8 & self.free
    }
    
    pub fn get_wpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.w_pieces[1] & RANK_2) << 8 & self.free;
        sgl_push << 8 & self.free
    }
    
    pub fn get_wpawn_left_captures(&self) -> u64 {
        (self.w_pieces[1] ^ FILE_A) << 7 & self.b_pieces[0]
    }
    
    pub fn get_wpawn_right_captures(&self) -> u64 {
        (self.w_pieces[1] ^ FILE_H) << 9 & self.b_pieces[0]
    }

    pub fn get_wpawn_left_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pieces[1] ^ FILE_A) << 7 & self.en_passant_target_sq
    }

    pub fn get_wpawn_right_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pieces[1] ^ FILE_H) << 9 & self.en_passant_target_sq
    }
    
    pub fn get_bpawn_sgl_pushes(&self) -> u64 {
        self.b_pieces[1] >> 8 & self.free
    }
    
    pub fn get_bpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.b_pieces[1] & RANK_7) >> 8 & self.free;
        sgl_push >> 8 & self.free
    }
    
    pub fn get_bpawn_left_captures(&self) -> u64 {
        (self.b_pieces[1] ^ FILE_A) >> 9 & self.w_pieces[0]
    }

    pub fn get_bpawn_right_captures(&self) -> u64 {
        (self.b_pieces[1] ^ FILE_H) >> 7 & self.w_pieces[0]
    }
    
    pub fn get_bpawn_left_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pieces[1] ^ FILE_A) >> 9 & self.en_passant_target_sq
    }

    pub fn get_bpawn_right_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pieces[1] ^ FILE_H) >> 7 & self.en_passant_target_sq
    }

}

/// Methods to generate unsafe squares for the king.
impl Position {

    pub fn get_unsafe_squares_for(&self, color: Color, maps: &Maps) -> (u64, u64) {
        let piece_set;
        let occ;
        // Remove the king from the occupancy bitboard for sliding piece move 
        // generation to prevent the king from blocking other unsafe squares
        match color {
            Color::White => {
                piece_set = &self.b_pieces;
                occ = self.occ ^ self.w_pieces[Piece::King as usize];
            },
            Color::Black => {
                piece_set = &self.w_pieces;
                occ = self.occ ^ self.b_pieces[Piece::King as usize];
            }
        }
        let mut unsafe_squares: u64 = 0;
        let mut attackers: u64 = 0;
        let king = piece_set[Piece::King as usize];
        // Pawn captures
        if matches!(color, Color::White) {
            unsafe_squares |= bittools::sout_east(piece_set[Piece::Pawn as usize]);
            unsafe_squares |= bittools::sout_west(piece_set[Piece::Pawn as usize]);
            attackers |= bittools::nort_east(king) & piece_set[Piece::Pawn as usize];
            attackers |= bittools::nort_west(king) & piece_set[Piece::Pawn as usize];
        } else {
            unsafe_squares |= bittools::nort_east(piece_set[Piece::Pawn as usize]);
            unsafe_squares |= bittools::nort_west(piece_set[Piece::Pawn as usize]);
            attackers |= bittools::sout_east(king) & piece_set[Piece::Pawn as usize];
            attackers |= bittools::sout_west(king) & piece_set[Piece::Pawn as usize];
        }
        // Horizontal and vertical sliding pieces
        let hv_pieces = piece_set[Piece::Rook as usize] | piece_set[Piece::Queen as usize];
        for hv_piece in bittools::forward_scan(hv_pieces) {
            let rank_attacks = bittools::hyp_quint(occ, hv_piece, &maps.rank);
            let file_attacks = bittools::hyp_quint(occ, hv_piece, &maps.file);
            unsafe_squares |= rank_attacks;
            unsafe_squares |= file_attacks;
            // If the king is in the direct line of attack, add the hv_piece as
            // an attacker
            if king & (rank_attacks | file_attacks) != 0 {
                attackers |= hv_piece
            }
        }
        // Diagonal and antidiagonal sliding pieces
        let da_pieces = piece_set[Piece::Bishop as usize] | piece_set[Piece::Queen as usize];
        for da_piece in bittools::forward_scan(da_pieces) {
            let diag_attacks = bittools::hyp_quint(occ, da_piece, &maps.diag);
            let adiag_attacks = bittools::hyp_quint(occ, da_piece, &maps.adiag);
            unsafe_squares |= diag_attacks;
            unsafe_squares |= adiag_attacks;
            // If the king is in the direct line of attack, add the da_piece as
            // an attacker
            if king & (diag_attacks | adiag_attacks) != 0 {
                attackers |= da_piece
            }
        }
        // Knights
        unsafe_squares |= maps.dknight.get(&piece_set[Piece::Knight as usize]).unwrap();
        attackers |= maps.knight[bittools::ilsb(&king)] & piece_set[Piece::Knight as usize];
        // Kings
        unsafe_squares |= maps.king[bittools::ilsb(&piece_set[Piece::King as usize])];

        return (unsafe_squares, attackers);
    }

    pub fn get_color_at(&self, n: u64) -> Color {
        let color;
        if n & self.w_pieces[0] != 0 {
            color = Color::White
        } else if n & self.b_pieces[0] != 0 {
            color = Color::Black
        } else {
            panic!("Method Position.get_color_at could not locate the bit")
        }
        return color;
    }

    pub fn get_piece_at(&self, n: u64) -> Piece {
        let mut result = Piece::Any;
        for piece in Piece::iterator() {
            if (self.w_pieces[piece as usize] | self.b_pieces[piece as usize]) & n != 0 {
                result = piece;
                break
            }
        }
        if matches!(result, Piece::Any) {
            panic!("Method Position.get_piece_at could not locate the bit")
        }
        return result;
    }

    pub fn piece_at_is_slider(&self, n: u64) -> bool {
        matches!(self.get_piece_at(n), Piece::Rook | Piece::Bishop | Piece::Queen) 
    }

}

/// Methods to make and unmake a move
impl Position {

    pub fn make_move(&mut self, mv: &Move) {
        let f_pieces;
        let o_pieces;

        if self.white_to_move {
            f_pieces = &mut self.w_pieces;
            o_pieces = &mut self.b_pieces;
        } else {
            f_pieces = &mut self.b_pieces;
            o_pieces = &mut self.w_pieces;
        }
        // Free up src squares and occupy target squares
        self.occ ^= mv.src;
        self.free |= mv.src;
        f_pieces[0] ^= mv.src | mv.target;
        f_pieces[mv.moved_piece as usize] ^= mv.src | mv.target;
        self.occ |= mv.target;
        if mv.is_capture {
            o_pieces[mv.captured_piece as usize] ^= mv.target;
            o_pieces[0] ^= mv.target;
        }
        // Set en passant target sq if the move was a double pawn push
        if matches!(mv.moved_piece, Piece::Pawn) 
            && (((mv.src << 16) == mv.target) | ((mv.src >> 16) == mv.target)) {
                if self.white_to_move {
                    self.en_passant_target_sq = mv.src << 8;
                } else {
                    self.en_passant_target_sq = mv.src >> 8;
                }
        } else {
            self.en_passant_target_sq = 0;
        }
        // Set the clocks
        if mv.is_capture || matches!(mv.moved_piece, Piece::Pawn) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if !self.white_to_move {
            self.fullmove_clock += 1;
        }
    }

    pub fn unmake_move(&self, mv: &Move) {

    }
    
}
