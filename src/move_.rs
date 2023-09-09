use super::*;
use types::{MoveType, Piece};

/*
    Moves are encoded in an 16 bit integer.
    Bits 0-5 and 6-11 encode the source and target square, respectively.
    Bits 12-15 encode bitflags, included in enum discriminants:

    FLAGS
    -----
    0000 - quiet moves
    0001 - double pawn push
    0010 - king castle
    0011 - queen castle
    0100 - captures
    0101 - en passant capture
    0110 - NONE
    0111 - NONE
    1000 - knight promotion
    1001 - bishop promotion
    1010 - rook promotion
    1011 - queen promotion
    1100 - knight-promo capture
    1101 - bishop-promo capture
    1110 - rook-promo capture
    1111 - queen-promo capture

    x1xx - capture flag
    1xxx - promotion flag
*/

#[derive(Clone, Copy)]
pub struct Move(pub u16);

impl Move {
    pub fn null() -> Move {
        return Move(0);
    }

    pub fn from_uint16(word: u16) -> Move {
        return Move(word);
    }

    pub fn encode(from: BitBoard, to: BitBoard, movetype: MoveType) -> Self {
        return Self(from.to_square_uint16() | (to.to_square_uint16() << 6) | movetype as u16);
    }

    /// Decode the target into a one bit bitmask
    pub fn to(&self) -> BitBoard {
        const TARGET_BITS: u16 = 0x0fc0;
        BitBoard::from_square(((self.0 & TARGET_BITS) >> 6).into())
    }

    /// Decode the source into a one bit bitmask
    pub fn from(&self) -> BitBoard {
        const SOURCE_BITS: u16 = 0x003f;
        BitBoard::from_square((self.0 & SOURCE_BITS).into())
    }

    /// Decode the type of move
    pub fn movetype(&self) -> MoveType {
        use MoveType::*;
        const FLAG_BITS: u16 = 0xf000;

        match self.0 & FLAG_BITS {
            0x0000 => Quiet,
            0x1000 => DoublePawnPush,
            0x2000 => ShortCastle,
            0x3000 => LongCastle,
            0x4000 => Capture,
            0x5000 => EnPassant,
            0x8000 => KnightPromotion,
            0x9000 => BishopPromotion,
            0xa000 => RookPromotion,
            0xb000 => QueenPromotion,
            0xc000 => KnightPromotionCapture,
            0xd000 => BishopPromotionCapture,
            0xe000 => RookPromotionCapture,
            0xf000 => QueenPromotionCapture,
            _ => panic!("invalid bitflag"),
        }
    }

    pub fn is_quiet(&self) -> bool {
        self.0 & MoveType::Quiet as u16 == 0
    }

    /// Decode if the move encodes a capture of any sort
    pub fn is_capture(&self) -> bool {
        const CAPTURE_FLAG: u16 = 0x4000;
        return self.0 & CAPTURE_FLAG != 0;
    }

    /// Decode if the move encodes a promotion of any sort
    pub fn is_promotion(&self) -> bool {
        const PROMO_FLAG: u16 = 0x8000;
        return self.0 & PROMO_FLAG != 0;
    }

    /// Is the move a null move
    pub fn is_null(&self) -> bool {
        return self.0 == 0;
    }

    /// What kind of promotion is encoded
    pub fn promotion_piecetype(&self) -> Option<Piece> {
        const PT_FLAG: u16 = 0x3000;
        debug_assert!(self.is_promotion());
        match self.0 & PT_FLAG {
            0x0000 => Some(Piece::Knight),
            0x1000 => Some(Piece::Bishop),
            0x2000 => Some(Piece::Rook),
            0x3000 => Some(Piece::Queen),
            _ => None,
        }
    }

    pub fn to_algebraic(&self) -> String {
        let from = self.from().to_algebraic();
        let to = self.to().to_algebraic();

        let promo_pt = if self.is_promotion() {
            match self.promotion_piecetype().expect("is_promotion check") {
                Piece::Rook => "r",
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Queen => "q",
                _ => "",
            }
        } else {
            ""
        };

        return format!("{from}{to}{promo_pt}");
    }
}

impl std::cmp::PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
