/// Contains the functions required to parse a FEN string into a position

use std::collections::HashMap;

use super::*;
use crate::d;

/// Initialise a set of bitboards for white and black pieces from the portion
/// of the FEN string representing the board. Outputs two arrays, representing
/// black and white pieces, indexed by the discriminants of the Pieces enum. 
pub fn bitboards(board: &str) -> ([u64; 7], [u64; 7]) {
    let mappings = HashMap::from([
        ('P', d!(Piece::Pawn)),
        ('R', d!(Piece::Rook)),
        ('N', d!(Piece::Knight)),
        ('B', d!(Piece::Bishop)),
        ('Q', d!(Piece::Queen)),
        ('K', d!(Piece::King)),
    ]);
    let mut w_pieces: [u64; 7] = [0; 7];
    let mut b_pieces: [u64; 7] = [0; 7];
    // Split the FEN string at "/"
    let mut split_board: Vec<&str> = board.split("/").collect();
    assert!(split_board.len() == 8);
    // Reverse vector so that 0 index is now at square A1
    split_board.reverse();
    let rev_board = &split_board.join("")[..];
    let mut i = 0;
    for mut char in rev_board.chars() {
        let mask: u64 = 1 << i;
        if char.is_alphabetic() {
            // If the character is alphabetic, then it represents a piece;
            // populate the relevant bitboard
            let board_to_modify;
            if char.is_uppercase() {
                board_to_modify = &mut w_pieces;
            } else {
                board_to_modify = &mut b_pieces;
                char.make_ascii_uppercase();
            }
            board_to_modify[d!(Piece::Any)] |= mask;
            match mappings.get(&char) {
                Some(&i) => board_to_modify[i] |= mask,
                None => panic!("Invalid character {} in FEN", char),
            }
            i += 1;
        } else {
            assert!(char.is_numeric());
            // Character represents empty squares so skip over the matching
            // number of index positions.
            i += char.to_digit(10).unwrap();
        }
    }
    assert!(i == 64);
    return (w_pieces, b_pieces);
}

pub fn white_to_move(code: &str) -> bool {
    assert!(code == "w" || code == "b");
    return code == "w";
}

/// Return the castling rights of a position from as a tuple of
/// (w_kingside, b_kingside, w_queenside b_queenside)
pub fn castling_rights(code: &str) -> (bool, bool, bool, bool) {
    return (code.contains("K"), code.contains("k"),
            code.contains("Q"), code.contains("q"));
}

pub fn en_passant(algebraic: &str) -> u64 {
    let target_square: u64;
    let epts: Vec<char> = algebraic.chars().collect();
    if epts[0] != '-' {
        assert!(epts.len() == 2);
        assert!(epts[0].is_alphabetic());
        let file = epts[0] as u8;
        assert!(epts[1].is_numeric());
        let rank = epts[1] as u8;
        target_square = 1 << ((file - ASCIIBases::LowerA as u8)
            + (rank - ASCIIBases::Zero as u8 - 1) * 8);
    } else {
        target_square = EMPTY_BB;
    }
    return target_square;
}