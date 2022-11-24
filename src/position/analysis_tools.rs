/// Module containing functions to extract information from a position

use crate::{common::*, d};
use crate::common::bittools as bt;
use crate::global::maps::Maps;
use super::{Position, PieceSet};

/// Possible squares the white pawns can push to
pub fn w_pawn_sgl_pushes(pos: &Position) -> u64 {
    bt::north_one(
        pos.w_pieces.pawn
    ) & pos.free
}

/// Possible squares the white pawns can double push to
pub fn w_pawn_dbl_pushes(pos: &Position) -> u64 {
    let sgl_push: u64 = bt::north_one(
        pos.w_pieces.pawn & RANK_2
    ) & pos.free;
    bt::north_one(sgl_push) & pos.free
}

/// Possible squares the white pawns can capture left
pub fn w_pawn_left_captures(pos: &Position) -> u64 {
    bt::nort_west(
        pos.w_pieces.pawn
    ) & pos.b_pieces.pawn
}

/// Possible squares the white pawns can capture right
pub fn w_pawn_right_captures(pos: &Position) -> u64 {
    bt::nort_east(
        pos.w_pieces.pawn
    ) & pos.b_pieces.pawn
}

/// Possible squares the black pawns can push to
pub fn b_pawn_sgl_pushes(pos: &Position) -> u64 {
    bt::south_one(
        pos.b_pieces.pawn
    ) & pos.free
}

/// Possible squares the black pawns can double push to
pub fn b_pawn_dbl_pushes(pos: &Position) -> u64 {
    let sgl_push: u64 = bt::south_one(
        pos.b_pieces.pawn & RANK_7
    ) & pos.free;
    bt::south_one(sgl_push) & pos.free
}

/// Possible squares the black pawns can capture left
pub fn b_pawn_left_captures(pos: &Position) -> u64 {
    bt::sout_west(
        pos.b_pieces.pawn
    ) & pos.w_pieces.pawn
}

/// Possible squares the black pawns can capture right
pub fn b_pawn_right_captures(pos: &Position) -> u64 {
    bt::sout_east(
        pos.b_pieces.pawn
    ) & pos.w_pieces.pawn
}


pub fn w_pawn_target_gen_funcs() -> [fn(&Position) -> u64; 4] {
    [w_pawn_sgl_pushes, w_pawn_dbl_pushes, 
     w_pawn_left_captures, w_pawn_right_captures]
}

pub fn b_pawn_target_gen_funcs() -> [fn(&Position) -> u64; 4] {
    [b_pawn_sgl_pushes, b_pawn_dbl_pushes,
     b_pawn_left_captures, b_pawn_right_captures]
}

pub fn w_pawn_src_gen_funcs() -> [fn(u64) -> u64; 4] {
    [bt::south_one, bt::south_two, bt::sout_east, bt::sout_west]
}

pub fn b_pawn_src_gen_funcs() -> [fn(u64) -> u64; 4] {
    [bt::north_one, bt::north_two, bt::nort_east, bt::nort_west]
}


/// White pawns able to capture en passant
pub fn w_pawn_en_passant(pos: &Position) -> u64 {
    (bt::sout_west(pos.en_passant_target_sq) 
        | bt::sout_east(pos.en_passant_target_sq))
        & pos.w_pieces.pawn
        & RANK_5
}

/// Black pawns able to capture en passant
pub fn b_pawn_en_passant(pos: &Position) -> u64 {
    (bt::nort_west(pos.en_passant_target_sq)
    | bt::nort_east(pos.en_passant_target_sq))
    & pos.b_pieces.pawn
    & RANK_4
}

pub fn w_pawn_left_attack_sqs(pos: &Position) -> u64 {
    bt::nort_west(pos.w_pieces.pawn)
}

pub fn w_pawn_right_attack_sqs(pos: &Position) -> u64 {
    bt::nort_east(pos.w_pieces.pawn)
}

pub fn b_pawn_left_attack_sqs(pos: &Position) -> u64 {
    bt::sout_east(pos.b_pieces.pawn)
}

pub fn b_pawn_right_attack_sqs(pos: &Position) -> u64 {
    bt::sout_west(pos.b_pieces.pawn)
}

pub fn knight_attack_sqs(piece_set: &PieceSet, maps: &Maps) -> u64 {
    *maps.dknight.get(&piece_set.knight).unwrap()
}

pub fn king_attack_sqs(piece_set: &PieceSet, maps: &Maps) -> u64 {
    maps.king[bt::ilsb(&piece_set.king)]
}

/// Get all the squares the opponent pieces are attacking and all the opponent
/// pieces that are checking the king
pub fn find_unsafe_squares_and_checkers_for(
    color: &Color, pos: &Position, maps: &Maps
) -> (u64, u64) {
    // Initialise variables based on which color we want to find the unsafe
    // squares and checkers for
    let their_pieces;
    let our_pieces;
    let pawn_attack_funcs: [fn(&Position) -> u64; 2];
    let find_pawn_checker_funcs: [fn(u64) -> u64; 2];
    if matches!(color, Color::White) {
        our_pieces = &pos.w_pieces;
        their_pieces = &pos.b_pieces;
        pawn_attack_funcs = [b_pawn_left_attack_sqs, b_pawn_right_attack_sqs];
        find_pawn_checker_funcs = [bt::nort_east, bt::nort_west];
    } else {
        our_pieces = &pos.b_pieces;
        their_pieces = &pos.w_pieces;
        pawn_attack_funcs = [w_pawn_left_attack_sqs, w_pawn_right_attack_sqs];
        find_pawn_checker_funcs = [bt::sout_east, bt::sout_west];
    }
    let mut unsafe_squares: u64 = EMPTY_BB;
    let mut checkers: u64 = EMPTY_BB;
    // Remove the king from the occupancy bitboard for sliding piece move
    // generation to prevent the king from blocking other unsafe squares
    let occ = pos.occ ^ our_pieces.king;

    // Calculate pawn attacks
    find_pawn_attack_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, pawn_attack_funcs,
        find_pawn_checker_funcs, pos, their_pieces, our_pieces
    );
    // Calculate attacks in horizontal and vertical directions
    find_hv_unsafe_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces,
        our_pieces, maps, occ
    );
    // Calculate attacks in the diagonal and anti-diagonal directions
    find_da_unsafe_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces,
        our_pieces, maps, occ
    );
    // Calculate knight attacks
    find_knight_attack_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces, our_pieces, maps
    );
    // Calculate king attacks
    unsafe_squares |= king_attack_sqs(their_pieces, maps);

    return (unsafe_squares, checkers)
}

fn find_pawn_attack_squares_and_checkers(
    unsafe_squares: &mut u64,
    checkers: &mut u64,
    pawn_attack_funcs: [fn(&Position) -> u64; 2],
    find_pawn_checker_funcs: [fn(u64) -> u64; 2],
    pos: &Position,
    their_pieces: &PieceSet,
    our_pieces: &PieceSet,
) {
    for i in 0..2 {
        *unsafe_squares |= pawn_attack_funcs[i](pos);
        *checkers |= find_pawn_checker_funcs[i](our_pieces.king) 
            & their_pieces.pawn
    }
}

fn find_hv_unsafe_squares_and_checkers(
    unsafe_squares: &mut u64,
    checkers: &mut u64,
    their_pieces: &PieceSet,
    our_pieces: &PieceSet,
    maps: &Maps,
    occ: u64
) {
    let hv_pieces = their_pieces.rook | their_pieces.queen;
    for hv_piece in bt::forward_scan(hv_pieces) {
        let hv_attacks = bt::hv_hyp_quint(occ, hv_piece, maps);
        *unsafe_squares |= hv_attacks;
        if our_pieces.king & hv_attacks != EMPTY_BB {
            *checkers |= hv_piece
        }
    }
}

fn find_da_unsafe_squares_and_checkers(
    unsafe_squares: &mut u64,
    checkers: &mut u64,
    their_pieces: &PieceSet,
    our_pieces: &PieceSet,
    maps: &Maps,
    occ: u64
) {
    let da_pieces = their_pieces.bishop | their_pieces.queen;
    for da_piece in bt::forward_scan(da_pieces) {
        let da_attacks = bt::da_hyp_quint(occ, da_piece, maps);
        *unsafe_squares |= da_attacks;
        if our_pieces.king & da_attacks != EMPTY_BB {
            *checkers |= da_piece
        }
    }
}

fn find_knight_attack_squares_and_checkers(
    unsafe_squares: &mut u64,
    checkers: &mut u64,
    their_pieces: &PieceSet,
    our_pieces: &PieceSet,
    maps: &Maps
) {
    for knight in bt::forward_scan(their_pieces.knight) {
        let attacks = maps.knight[bt::ilsb(&knight)];
        *unsafe_squares |= attacks;
        if our_pieces.king & attacks != EMPTY_BB {
            *checkers |= knight
        }
    }
}

/// Get the colour at a particular square
pub fn get_color_at(pos: &Position, n: u64) -> Color {
    assert!(n.count_ones() == 1);
    let color;
    if n & pos.w_pieces.any != EMPTY_BB {
        color = Color::White
    } else if n & pos.b_pieces.any != EMPTY_BB {
        color = Color::Black
    } else {
        panic!(
            "Function get_color_at could not locate the requested bit {}",
            n.trailing_zeros()
        )
    }
    return color;
}

/// Identify which piece is a particular position
pub fn get_piece_at(pos: &Position, n: u64) -> Piece {
    assert!(n.count_ones() == 1);
    let mut result = Piece::Any;
    let w_piece_array = pos.w_pieces.as_array();
    let b_piece_array = pos.b_pieces.as_array();
    for piece in Piece::iter_pieces() {
        if (
            w_piece_array[d!(piece)] | b_piece_array[d!(piece)]
        ) & n != EMPTY_BB {
            result = piece;
            return result;
        }
    }
    if matches!(result, Piece::Any) {
        panic!(
            "Function get_piece_at could not locate the requested bit {}",
            n.trailing_zeros()
        )
    }
    return result;
}

pub fn piece_at_is_slider(pos: &Position, n: u64) -> bool {
    matches!(
        get_piece_at(pos, n),
        Piece::Rook | Piece::Bishop | Piece::Queen
    ) 
}


pub fn get_pinned_pieces_for(pos: &Position, color: &Color, maps: &Maps) -> u64 {
    let their_pieces;
    let our_pieces;
    match color {
        Color::White => {
            their_pieces = &pos.b_pieces;
            our_pieces = &pos.w_pieces;
        }
        Color::Black => {
            their_pieces = &pos.w_pieces;
            our_pieces = &pos.b_pieces;
        }
    }
    let king = our_pieces.king;
    let mut pinned_pieces: u64 = 0;
    let king_h_rays = bittools::hyp_quint(pos.occ, king, &maps.rank);
    let king_v_rays = bittools::hyp_quint(pos.occ, king, &maps.file);
    let king_d_rays = bittools::hyp_quint(pos.occ, king, &maps.diag);
    let king_a_rays = bittools::hyp_quint(pos.occ, king, &maps.adiag);

    let hv_pieces = their_pieces.rook | their_pieces.queen;
    // Calculate horizontal and vertical pins
    for hv_piece in bittools::forward_scan(hv_pieces) {
        let h_piece_ray = bittools::hyp_quint(pos.occ, hv_piece, &maps.rank);
        // If the king and opponent piece rays align on the same friendly
        // piece, the piece must be pinned along the combined ray
        if h_piece_ray & king_h_rays & our_pieces.any != 0 {
            pinned_pieces |= hv_piece;
        }
        let v_piece_ray = bittools::hyp_quint(pos.occ, hv_piece, &maps.file);
        if v_piece_ray & king_v_rays & our_pieces.any != 0 {
            pinned_pieces |= hv_piece;
        }
    }
    // Calculate diagonal and antidiagonal pins
    let da_pieces = their_pieces.bishop | their_pieces.queen;
    for da_piece in bittools::forward_scan(da_pieces) {
        let d_piece_ray = bittools::hyp_quint(pos.occ, da_piece, &maps.diag);
        if d_piece_ray & king_d_rays & our_pieces.any != 0 {
            pinned_pieces |= da_piece;
        }
        let a_piece_ray = bittools::hyp_quint(pos.occ, da_piece, &maps.adiag);
        if a_piece_ray & king_a_rays & our_pieces.any != 0 {
            pinned_pieces |= da_piece;
        }
    }
    return pinned_pieces

}
