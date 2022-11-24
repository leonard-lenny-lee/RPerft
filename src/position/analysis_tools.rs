/// Module containing functions to extract information from a position

use crate::{common::*, d};
use crate::common::bittools as bt;
use super::Position;

/// Possible squares the white pawns can push to
pub fn w_pawn_sgl_pushes(pos: &Position) -> u64 {
    bt::north_one(
        pos.w_pieces[d!(Piece::Pawn)]
    ) & pos.free
}

/// Possible squares the white pawns can double push to
pub fn w_pawn_dbl_pushes(pos: &Position) -> u64 {
    let sgl_push: u64 = bt::north_one(
        pos.w_pieces[d!(Piece::Pawn)] & RANK_2
    ) & pos.free;
    bt::north_one(sgl_push) & pos.free
}

/// Possible squares the white pawns can capture left
pub fn w_pawn_left_captures(pos: &Position) -> u64 {
    bt::nort_west(
        pos.w_pieces[d!(Piece::Pawn)]
    ) & pos.b_pieces[d!(Piece::Any)]
}

/// Possible squares the white pawns can capture right
pub fn w_pawn_right_captures(pos: &Position) -> u64 {
    bt::nort_east(
        pos.w_pieces[d!(Piece::Pawn)]
    ) & pos.b_pieces[d!(Piece::Any)]
}

/// Possible squares the black pawns can push to
pub fn b_pawn_sgl_pushes(pos: &Position) -> u64 {
    bt::south_one(
        pos.b_pieces[d!(Piece::Pawn)]
    ) & pos.free
}

/// Possible squares the black pawns can double push to
pub fn b_pawn_dbl_pushes(pos: &Position) -> u64 {
    let sgl_push: u64 = bt::south_one(
        pos.b_pieces[d!(Piece::Pawn)] & RANK_7
    ) & pos.free;
    bt::south_one(sgl_push) & pos.free
}

/// Possible squares the black pawns can capture left
pub fn b_pawn_left_captures(pos: &Position) -> u64 {
    bt::sout_west(
        pos.b_pieces[d!(Piece::Pawn)]
    ) & pos.w_pieces[d!(Piece::Any)]
}

/// Possible squares the black pawns can capture right
pub fn b_pawn_right_captures(pos: &Position) -> u64 {
    bt::sout_east(
        pos.b_pieces[d!(Piece::Pawn)]
    ) & pos.w_pieces[d!(Piece::Any)]
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
        & pos.w_pieces[d!(Piece::Pawn)]
        & RANK_5
}

/// Black pawns able to capture en passant
pub fn b_pawn_en_passant(pos: &Position) -> u64 {
    (bt::nort_west(pos.en_passant_target_sq)
    | bt::nort_east(pos.en_passant_target_sq))
    & pos.b_pieces[d!(Piece::Pawn)]
    & RANK_4
}