/// Module containing functions to extract information from a position

use super::*;
use global::maps::Maps;
use crate::{disc, engine::common::bittools as bt};

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
    find_hv_unsafe_squares(&mut unsafe_squares, pos, occ);
    // Calculate attacks in the diagonal and anti-diagonal directions
    find_ad_unsafe_squares(&mut unsafe_squares, pos, occ);
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
    occ: u64
) {
    let pieces = pos.their_pieces().rook | pos.their_pieces().queen;
    let attacks = bt::rook_attacks(pieces, occ);
    *unsafe_squares |= attacks;
}

/// Find diagonal and antidiagonal unsafe squares
fn find_ad_unsafe_squares(
    unsafe_squares: &mut u64,
    pos: &Position,
    occ: u64
) {
    let pieces = pos.their_pieces().bishop | pos.their_pieces().queen;
    let attacks = bt::bishop_attacks(pieces, occ);
    *unsafe_squares |= attacks;
}

/// Find knight attack squares
fn find_knight_attack_squares(
    unsafe_squares: &mut u64,
    pos: &Position,
    maps: &Maps
) {
    let mut knights = pos.their_pieces().knight;
    while knights != 0 {
        let knight = bt::ilsb(knights);
        let attacks = maps.knight[knight];
        *unsafe_squares |= attacks;
        knights ^= 1 << knight;
    }
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
        if their_piece_array[disc!(piece)] & n != EMPTY_BB {
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
    let mut pinned_pieces: u64 = EMPTY_BB;

    // Calculate the rays from the king
    let king_rays = calculate_king_rays(pos, our_pieces.king, maps);
    let hv_pieces = their_pieces.rook | their_pieces.queen;
    let da_pieces = their_pieces.bishop | their_pieces.queen;

    // Calculate horizontal pins
    let h_attacks = bt::rank_attacks(hv_pieces, pos.data.occ);
    pinned_pieces |= h_attacks & king_rays[0];

    // Calculate vertical pins
    let v_attacks = bt::file_attacks(hv_pieces, pos.data.occ);
    pinned_pieces |= v_attacks & king_rays[1];

    // Calculate diagonal pins
    let d_attacks = bt::diag_attacks(da_pieces, pos.data.occ);
    pinned_pieces |= d_attacks & king_rays[2];

    // Calculate antidiagonal pins
    let a_attacks = bt::adiag_attacks(da_pieces, pos.data.occ);
    pinned_pieces |= a_attacks & king_rays[3];
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