/// Module containing methods to extract information from a position
// TODO Implement State Pattern for positions for white to move and black to move
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

/// Collate all the relevant functions for generating the white pawn targets
pub fn w_pawn_target_gen_funcs() -> [fn(&Position) -> u64; 4] {
    [w_pawn_sgl_pushes, w_pawn_dbl_pushes, 
     w_pawn_left_captures, w_pawn_right_captures]
}

/// Collate all the relevant functions for generating the black pawn targets
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

/// White pawns left attack squares
pub fn w_pawn_left_attack_sqs(pos: &Position) -> u64 {
    bt::nort_west(pos.w_pieces.pawn)
}

/// White pawns right attack squares
pub fn w_pawn_right_attack_sqs(pos: &Position) -> u64 {
    bt::nort_east(pos.w_pieces.pawn)
}

/// Black pawns left attack squares
pub fn b_pawn_left_attack_sqs(pos: &Position) -> u64 {
    bt::sout_east(pos.b_pieces.pawn)
}

/// Black pawns right attack squares
pub fn b_pawn_right_attack_sqs(pos: &Position) -> u64 {
    bt::sout_west(pos.b_pieces.pawn)
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
    find_hor_ver_unsafe_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces,
        our_pieces, maps, occ
    );
    // Calculate attacks in the diagonal and anti-diagonal directions
    find_diag_adiag_unsafe_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces,
        our_pieces, maps, occ
    );
    // Calculate knight attacks
    find_knight_attack_squares_and_checkers(
        &mut unsafe_squares, &mut checkers, their_pieces, our_pieces, maps
    );
    // Calculate king attacks
    unsafe_squares |= maps.retreive_king_map(their_pieces.king);

    return (unsafe_squares, checkers)
}

/// Function to add pawn attack squares and checkers to the master bitboards
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

fn find_hor_ver_unsafe_squares_and_checkers(
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

fn find_diag_adiag_unsafe_squares_and_checkers(
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
        let attacks = maps.retrieve_knight_map(knight);
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

/// Identify if the piece at the specified square is a sliding piece
pub fn piece_at_is_slider(pos: &Position, n: u64) -> bool {
    matches!(
        get_piece_at(pos, n),
        Piece::Rook | Piece::Bishop | Piece::Queen
    ) 
}

/// Identify which pieces are pinned for a particular color in a position
pub fn get_pinned_pieces_for(
    pos: &Position,
    color: &Color,
    maps: &Maps
) -> u64 {
    // Initialise variables
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
    let king_bb = our_pieces.king;
    let our_pieces_not_king = our_pieces.any ^ our_pieces.king;
    let mut pinned_pieces: u64 = EMPTY_BB;

    // Calculate the rays from the king
    let king_rays = calculate_king_rays(pos, king_bb, maps);

    // Calculate horizontal and vertical pins
    let hv_pieces = their_pieces.rook | their_pieces.queen;
    find_pins_for_direction(
        &mut pinned_pieces, hv_pieces, pos, 
        our_pieces_not_king, king_rays[0], &maps.rank
    );
    find_pins_for_direction(
        &mut pinned_pieces, hv_pieces, pos,
        our_pieces_not_king, king_rays[1], &maps.file
    );
    // Calculate diagonal and antidiagonal pins
    let da_pieces = their_pieces.bishop | their_pieces.queen;
    find_pins_for_direction(
        &mut pinned_pieces, da_pieces, pos,
        our_pieces_not_king, king_rays[2], &maps.diag
    );
    find_pins_for_direction(
        &mut pinned_pieces, da_pieces, pos,
        our_pieces_not_king, king_rays[3], &maps.adiag
    );
    return pinned_pieces

}

/// Calculate the rays from the king along the four axes which the piece may 
/// be pinned to. Returns an array of bitboards representing the horizontal,
/// vertical, diagonal and antidiagonal rays, respectively
fn calculate_king_rays(pos: &Position, king_bb: u64, maps: &Maps) -> [u64; 4] {
    let h_rays = bt::hyp_quint(pos.occ, king_bb, &maps.rank);
    let v_rays = bt::hyp_quint(pos.occ, king_bb, &maps.file);
    let d_rays = bt::hyp_quint(pos.occ, king_bb, &maps.diag);
    let a_rays = bt::hyp_quint(pos.occ, king_bb, &maps.adiag);
    return [h_rays, v_rays, d_rays, a_rays]
}

/// Find the pieces pinned by pinning pieces in a certain direction. Note: the
/// direction of the king_ray MUST be the same as the direction of the rays to
/// be calculated for the pinning pieces (as specified by the maps provided)
fn find_pins_for_direction(
    pinned_pieces: &mut u64,
    pinning_pieces: u64,
    pos: &Position,
    our_pieces_not_king: u64,
    king_ray: u64,
    maps: &[u64; 64]
) {
    for pinning_piece in bt::forward_scan(pinning_pieces) {
        let ray = bt::hyp_quint(pos.occ, pinning_piece, maps);
        // If the king and opponent piece rays align on the same square and 
        // that piece is ours, it must be pinned
        *pinned_pieces |= ray & king_ray & our_pieces_not_king
    }
}