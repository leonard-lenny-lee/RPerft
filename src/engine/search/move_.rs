use super::*;
use position::{Position, analysis_tools};

pub struct Move {
    pub target: u64,
    pub src: u64,
    pub moved_piece: Piece,
    pub promotion_piece: Promotion,
    pub special_move_flag: SpecialMove,
    pub is_capture: bool,
    pub captured_piece: u8, // Store the discriminant
}

impl Move {
    pub fn new(
        target_sq: u64, src_sq: u64, moved_piece: Piece, 
        promotion_piece: Promotion, special_move_flag: SpecialMove, 
        pos: &Position
    ) -> Move {
        // Identify which piece has been captured
        let is_capture = pos.their_pieces().any & target_sq != EMPTY_BB;
        let mut captured_piece = 0;
        if is_capture {
            captured_piece = analysis_tools::get_their_piece_at(
                pos, target_sq
            )
        }
        return Move {
            target: target_sq,
            src: src_sq,
            moved_piece,
            promotion_piece,
            special_move_flag,
            is_capture,
            captured_piece,
        };
    }
}