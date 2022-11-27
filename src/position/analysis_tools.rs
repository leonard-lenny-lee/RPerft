/// Module containing functions to extract information from a position

use crate::{common::*, d};
use crate::common::bittools as bt;
use crate::global::maps::Maps;
use super::Position;

/// Get all the squares the opponent pieces are attacking in a position and 
/// the location of all the opponent pieces that are checking the king
pub fn find_unsafe_squares(pos: &Position, maps: &Maps) -> u64 {
    // Initialise variables
    let mut unsafe_squares: u64 = EMPTY_BB;
    // Remove our king from the occupancy bitboard for sliding piece move
    // generation to prevent the king from blocking other unsafe squares
    let occ = pos.data.occ ^ pos.our_pieces().king;
    // Calculate pawn attacks
    unsafe_squares |= pos.unsafe_squares_pawn();
    // Calculate attacks in horizontal and vertical directions
    find_hv_unsafe_squares(&mut unsafe_squares, pos, maps, occ);
    // Calculate attacks in the diagonal and anti-diagonal directions
    find_ad_unsafe_squares(&mut unsafe_squares, pos, maps, occ);
    // Calculate knight attacks
    find_knight_attack_squares(&mut unsafe_squares, pos, maps);
    // Calculate king attacks
    unsafe_squares |= maps.get_king_map(pos.their_pieces().king);
    return unsafe_squares
}

/// Find horizontal and vertical unsafe squares
fn find_hv_unsafe_squares(
    unsafe_squares: &mut u64,
    pos: &Position,
    maps: &Maps,
    occ: u64
) {
    let pieces = pos.their_pieces().rook | pos.their_pieces().queen;
    for piece in bt::forward_scan(pieces) {
        let attacks = bt::hv_hyp_quint(occ, piece, maps);
        *unsafe_squares |= attacks;
    }
}

/// Find diagonal and antidiagonal unsafe squares
fn find_ad_unsafe_squares(
    unsafe_squares: &mut u64,
    pos: &Position,
    maps: &Maps,
    occ: u64
) {
    let pieces = pos.their_pieces().bishop | pos.their_pieces().queen;
    for piece in bt::forward_scan(pieces) {
        let attacks = bt::da_hyp_quint(occ, piece, maps);
        *unsafe_squares |= attacks;
    }
}

/// Find knight attack squares
fn find_knight_attack_squares(
    unsafe_squares: &mut u64,
    pos: &Position,
    maps: &Maps
) {
    let knights = pos.their_pieces().knight;
    let attacks = maps.get_dknight_map(&knights);
    *unsafe_squares |= attacks;
}

pub fn find_checkers(pos: &Position, maps: &Maps) -> u64 {
    let mut checkers: u64 = EMPTY_BB;
    // Find checking pawns
    checkers |= pos.their_checking_pawns();
    // Find horizontal and vertical checkers
    find_hv_checkers(&mut checkers, pos, maps);
    // Find diagonal and antidiagonal checkers
    find_ad_checkers(&mut checkers, pos, maps);
    // Find knight checkers
    find_knight_checkers(&mut checkers, pos, maps);
    checkers
}

fn find_hv_checkers(checkers: &mut u64, pos: &Position, maps: &Maps) {
    let mut pseudo_attacks = bt::hv_hyp_quint(
        pos.data.occ, pos.our_pieces().king, maps
    );
    pseudo_attacks &= pos.their_pieces().rook | pos.their_pieces().queen;
    *checkers |= pseudo_attacks
}

fn find_ad_checkers(checkers: &mut u64, pos: &Position, maps: &Maps) {
    let mut pseudo_attacks = bt::da_hyp_quint(
        pos.data.occ, pos.our_pieces().king, maps
    );
    pseudo_attacks &= pos.their_pieces().bishop | pos.their_pieces().queen;
    *checkers |= pseudo_attacks
}

fn find_knight_checkers(checkers: &mut u64, pos: &Position, maps: &Maps) {
    let king = pos.our_pieces().king;
    let pseudo_attacks = maps.knight[bt::ilsb(king)];
    *checkers |= pseudo_attacks & pos.their_pieces().knight
}

/// Get the colour at a particular square
pub fn get_color_at(pos: &Position, n: u64) -> Color {
    assert!(n.count_ones() == 1);
    let color;
    if n & pos.data.w_pieces.any != EMPTY_BB {
        color = Color::White
    } else if n & pos.data.b_pieces.any != EMPTY_BB {
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
    let their_piece_array = pos.their_pieces().as_array();
    for piece in Piece::iter_pieces() {
        if their_piece_array[d!(piece)] & n != EMPTY_BB {
            return piece
        }
    }
    if true {
        panic!(
            "Function get_piece_at could not locate the requested bit {}",
            n.trailing_zeros()
        )
    }
    return Piece::Any;
}

/// Identify if the piece at the specified square is a sliding piece
pub fn their_piece_at_is_slider(pos: &Position, n: u64) -> bool {
    matches!(
        get_their_piece_at(pos, n),
        Piece::Rook | Piece::Bishop | Piece::Queen
    ) 
}

/// Identify which pieces are pinned for a particular color in a position
pub fn get_pinned_pieces_for(pos: &Position, maps: &Maps) -> u64 {
    // Initialise variables
    let our_pieces = pos.our_pieces();
    let their_pieces = pos.their_pieces();
    let our_pieces_not_king = our_pieces.any ^ our_pieces.king;
    let mut pinned_pieces: u64 = EMPTY_BB;

    // Calculate the rays from the king
    let king_rays = calculate_king_rays(pos, our_pieces.king, maps);
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
fn calculate_king_rays(pos: &Position, king: u64, maps: &Maps) -> [u64; 4] {
    let h_rays = bt::hyp_quint(
        pos.data.occ, king, &maps.rank
    );
    let v_rays = bt::hyp_quint(
        pos.data.occ, king, &maps.file
    );
    let d_rays = bt::hyp_quint(
        pos.data.occ, king, &maps.diag
    );
    let a_rays = bt::hyp_quint(
        pos.data.occ, king, &maps.adiag
    );
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
        let ray = bt::hyp_quint(
            pos.data.occ, pinning_piece, maps
        );
        // If the king and opponent piece rays align on the same square and 
        // that piece is ours, it must be pinned
        *pinned_pieces |= ray & king_ray & our_pieces_not_king
    }
}