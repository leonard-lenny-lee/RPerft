use super::*;
use position::Position;

const FROM_TO: u8 = 63;
const PROMOTION_FLAG: u8 = 128;
const CAPTURE_FLAG: u8 = 64;

pub struct Move {
    pub promotion_piece: Promotion,
    pub special_move_flag: SpecialMove,
    src: u8,
    target: u8,
    /*
        CODES
        -----
        0000 - quiet moves
        0001 - double pawn push
        0010 - king castle
        0011 - queen castle
        x1xx - capture flag
        0101 - en passant capture
        1xxx - promotion flag
        1x00 - knight promotion
        1x01 - bishop promotion
        1x10 - rook promotion
        1x11 - queen promotion
    */
}

impl Move {
    pub fn new(
        target: u64, src: u64, moved_piece: Piece, 
        promotion_piece: Promotion, special_move_flag: SpecialMove, 
        pos: &Position
    ) -> Move {
        // Identify which piece has been captured
        let is_capture = pos.their_pieces().any & target != EMPTY_BB;
        let mut captured_piece = 0;
        let mut src = Move::encode_src(src);
        let mut target = Move::encode_target(target);
        if is_capture {
            src |= CAPTURE_FLAG
        }
        return Move {
            promotion_piece,
            special_move_flag,
            src,
            target
        };
    }

    // pub fn new_quiet_move(target: u64, src: u64) -> Move {
    //     return Move {

    //     }
    // }

    fn encode_target(target: u64) -> u8 {
        bt::ilsb_u8(target)
    }

    fn encode_src(src: u64) -> u8 {
        bt::ilsb_u8(src)
    } 

    /// Decode the target into a one bit bitmask
    pub fn target(&self) -> u64 {
        1 << (self.target & FROM_TO)
    }

    /// Decode the source into a one bit bitmask
    pub fn src(&self) -> u64 {
        1 << (self.src & FROM_TO)
    }

    /// Decode if the piece is a capture
    pub fn is_capture(&self) -> bool {
        self.src & CAPTURE_FLAG != 0
    }

    /// Special move flag
    pub fn is_promotion(&self) -> bool {
        self.src & PROMOTION_FLAG != 0
    }

}