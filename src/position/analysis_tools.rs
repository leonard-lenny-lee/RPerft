/// Module containing methods to extract information from a position
// TODO Implement State Pattern for positions for white to move and black to move
use crate::{common::*, d};
use crate::common::bittools as bt;
use crate::global::maps::Maps;
use super::{Data, PieceSet, Position};

/// Get all the squares the opponent pieces are attacking and all the opponent
/// pieces that are checking the king
pub fn find_unsafe_squares_and_checkers_for(
    pos: &Data, maps: &Maps
) -> (u64, u64) {
    // Initialise variables based on which color we want to find the unsafe
    // squares and checkers for
    let their_pieces;
    let our_pieces;
    let pawn_attack_funcs: [fn(&Data) -> u64; 2];
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
    unsafe_squares |= maps.get_king_map(their_pieces.king);

    return (unsafe_squares, checkers)
}

/// Function to add pawn attack squares and checkers to the master bitboards
fn find_pawn_attack_squares_and_checkers(
    unsafe_squares: &mut u64,
    checkers: &mut u64,
    pawn_attack_funcs: [fn(&Data) -> u64; 2],
    find_pawn_checker_funcs: [fn(u64) -> u64; 2],
    pos: &Data,
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
        let attacks = maps.get_knight_map(knight);
        *unsafe_squares |= attacks;
        if our_pieces.king & attacks != EMPTY_BB {
            *checkers |= knight
        }
    }
}

/// Get the colour at a particular square
pub fn get_color_at(pos: &Data, n: u64) -> Color {
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

/// Identify which opponent piece is a particular position
pub fn get_their_piece_at(pos: &Position, n: u64) -> Piece {
    assert!(n.count_ones() == 1);
    let mut result = Piece::Any;
    let their_piece_array = pos.their_pieces().as_array();
    for piece in Piece::iter_pieces() {
        if their_piece_array[d!(piece)] & n != EMPTY_BB {
            return piece
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
pub fn piece_at_is_slider(pos: &Data, n: u64) -> bool {
    matches!(
        get_their_piece_at(pos, n),
        Piece::Rook | Piece::Bishop | Piece::Queen
    ) 
}

/// Identify which pieces are pinned for a particular color in a position
pub fn get_pinned_pieces_for(
    pos: &Data,
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
fn calculate_king_rays(pos: &Data, king_bb: u64, maps: &Maps) -> [u64; 4] {
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
    pos: &Data,
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