/// Contains the Position struct, which holds the all the bitboards and data
/// to describe the current position, as well as methods to derive other
/// bitboards required for move generation and evaluation

mod init;
pub mod analysis_tools;

use super::common::*;
use super::global::maps::Maps;
use crate::d;

#[derive(Clone, Copy)]
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
        // Initialise bitboards
        let board = split_fen[0];
        let (w_pieces, b_pieces) = init::bitboards(board);
        let occ = w_pieces[d!(Piece::Any)] | b_pieces[d!(Piece::Any)];
        let free = !occ;
        // Set white to move
        let white_to_move = init::white_to_move(split_fen[1]);
        // Set castling rights
        let (
            w_kingside_castle, b_kingside_castle,
            w_queenside_castle, b_queenside_castle
        ) = init::castling_rights(split_fen[2]);
        // Set en passant target square
        let en_passant_target_sq = init::en_passant(split_fen[3]);
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

/// Methods to generate maps required for filtering legal moves from all
/// pseudolegal moves
impl Position {

    pub fn get_unsafe_squares_for(&self, color: &Color, maps: &Maps) -> (u64, u64) {
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
        let mut unsafe_squares: u64 = EMPTY_BB;
        let mut attackers: u64 = EMPTY_BB;
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

    pub fn get_pinned_pieces_for(&self, color: &Color, maps: &Maps) -> u64 {
        let opp_piece_set;
        let fr_piece_set;
        match color {
            Color::White => {
                opp_piece_set = &self.b_pieces;
                fr_piece_set = &self.w_pieces;
            }
            Color::Black => {
                opp_piece_set = &self.w_pieces;
                fr_piece_set = &self.b_pieces;
            }
        }
        let king = fr_piece_set[Piece::King as usize];
        let mut pinned_pieces: u64 = 0;
        let king_h_rays = bittools::hyp_quint(self.occ, king, &maps.rank);
        let king_v_rays = bittools::hyp_quint(self.occ, king, &maps.file);
        let king_d_rays = bittools::hyp_quint(self.occ, king, &maps.diag);
        let king_a_rays = bittools::hyp_quint(self.occ, king, &maps.adiag);

        let hv_pieces = opp_piece_set[Piece::Rook as usize] | opp_piece_set[Piece::Queen as usize];
        // Calculate horizontal and vertical pins
        for hv_piece in bittools::forward_scan(hv_pieces) {
            let h_piece_ray = bittools::hyp_quint(self.occ, hv_piece, &maps.rank);
            // If the king and opponent piece rays align on the same friendly
            // piece, the piece must be pinned along the combined ray
            if h_piece_ray & king_h_rays & fr_piece_set[0] != 0 {
                pinned_pieces |= hv_piece;
            }
            let v_piece_ray = bittools::hyp_quint(self.occ, hv_piece, &maps.file);
            if v_piece_ray & king_v_rays & fr_piece_set[0] != 0 {
                pinned_pieces |= hv_piece;
            }
        }
        // Calculate diagonal and antidiagonal pins
        let da_pieces = opp_piece_set[Piece::Bishop as usize] | opp_piece_set[Piece::Queen as usize];
        for da_piece in bittools::forward_scan(da_pieces) {
            let d_piece_ray = bittools::hyp_quint(self.occ, da_piece, &maps.diag);
            if d_piece_ray & king_d_rays & fr_piece_set[0] != 0 {
                pinned_pieces |= da_piece;
            }
            let a_piece_ray = bittools::hyp_quint(self.occ, da_piece, &maps.adiag);
            if a_piece_ray & king_a_rays & fr_piece_set[0] != 0 {
                pinned_pieces |= da_piece;
            }
        }
        return pinned_pieces

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
        for piece in Piece::iter_pieces() {
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
