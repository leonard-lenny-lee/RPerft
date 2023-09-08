use super::*;

use position::Position;
use types::{MoveType, PieceType};

pub trait MoveList {
    fn len(&self) -> usize;

    fn add(&mut self, from: BitBoard, to: BitBoard, movetype: MoveType, position: &Position);

    fn add_promotions(&mut self, from: BitBoard, to: BitBoard, position: &Position) {
        for mt in types::PROMOTION_MOVE_TYPES {
            self.add(from, to, mt, position)
        }
    }

    fn add_promotion_captures(&mut self, from: BitBoard, to: BitBoard, position: &Position) {
        for mt in types::PROMOTION_CAPTURE_MOVE_TYPES {
            self.add(from, to, mt, position)
        }
    }
}

// "Move valuable victim, least valuable aggressor" is a simple move ordering
// strategy to play winning captures first
const MVV_LVA_TABLE: [[i16; 7]; 7] = {
    // Based on the increasing value of pieces: P->N->B->R->Q->K
    const VALUES: [i16; 7] = [0, 100, 400, 200, 300, 500, 600];

    let mut table = [[0; 7]; 7];
    let mut victim = 0;
    while victim < 7 {
        let mut attacker = 0;
        while attacker < 7 {
            table[victim][attacker] = VALUES[victim] - VALUES[attacker] / 10;
            attacker += 1;
        }
        victim += 1;
    }
    table
};

/// Scores moves as they are added and orders them to optimize pruning
pub struct OrderedList {
    pub moves: Vec<(Move, i16)>,
    table_move: Move,
    killer_moves: [Move; 2],
    history_table: *const search::HistoryTable,
}

impl MoveList for OrderedList {
    fn len(&self) -> usize {
        self.moves.len()
    }

    fn add(&mut self, from: BitBoard, to: BitBoard, movetype: MoveType, position: &Position) {
        const PAWN_ID: usize = PieceType::Pawn as usize;

        const HASH_SCORE: i16 = 1000;
        const KILLER_SCORE_ONE: i16 = 50;
        const KILLER_SCORE_TWO: i16 = 45;

        let mv = Move::encode(from, to, movetype);

        // Hash moves are given the highest score
        if mv == self.table_move {
            self.moves.push((mv, HASH_SCORE));
            return;
        }

        // Use MVV-LLA to score captures
        let score = if mv.is_capture() {
            let victim = match position.them.piecetype_at(to) {
                Some(pt) => pt as usize,
                None => PAWN_ID, // EP is the only case where the target sq is empty.
            };

            let attacker = position.us.piecetype_at(from).unwrap() as usize;
            MVV_LVA_TABLE[victim][attacker]
        }
        // Score quiet moves with heuristics
        else {
            if mv == self.killer_moves[0] {
                KILLER_SCORE_ONE // Primary killer
            } else if mv == self.killer_moves[1] {
                KILLER_SCORE_TWO // Secondary killer
            } else {
                // History heuristic
                unsafe { (*self.history_table).get(mv.from(), mv.to()) as i16 }
            }
        };

        self.moves.push((mv, score));
    }
}

impl OrderedList {
    pub fn new(
        table_move: Move,
        killer_moves: [Move; 2],
        history_table: *const search::HistoryTable,
    ) -> Self {
        Self {
            moves: Vec::with_capacity(45),
            table_move,
            killer_moves,
            history_table,
        }
    }

    pub fn sort_by_score(&mut self) {
        self.moves.sort_by_key(|mv| std::cmp::Reverse(mv.1));
    }
}

impl std::ops::Index<usize> for OrderedList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.moves.index(index).0
    }
}

pub struct UnorderedList(pub Vec<Move>);

impl MoveList for UnorderedList {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn add(&mut self, from: BitBoard, to: BitBoard, mt: MoveType, _pos: &Position) {
        self.0.push(Move::encode(from, to, mt));
    }
}

impl UnorderedList {
    pub fn new() -> Self {
        // Based on an average branching factor of 35
        Self(Vec::with_capacity(45))
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.0.iter()
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<Move> {
        self.0.pop()
    }

    pub fn find(&self, mv: String) -> Option<Move> {
        for m in self.0.iter() {
            if mv == m.to_algebraic() {
                return Some(*m);
            }
        }
        return None;
    }
}

impl std::ops::Index<usize> for UnorderedList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

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
    pub fn promotion_piecetype(&self) -> Option<PieceType> {
        const PT_FLAG: u16 = 0x3000;
        debug_assert!(self.is_promotion());
        match self.0 & PT_FLAG {
            0x0000 => Some(PieceType::Knight),
            0x1000 => Some(PieceType::Bishop),
            0x2000 => Some(PieceType::Rook),
            0x3000 => Some(PieceType::Queen),
            _ => None,
        }
    }

    pub fn to_algebraic(&self) -> String {
        let from = self.from().to_algebraic();
        let to = self.to().to_algebraic();

        let promo_pt = if self.is_promotion() {
            match self.promotion_piecetype().expect("is_promotion check") {
                PieceType::Rook => "r",
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Queen => "q",
                _ => "",
            }
        } else {
            ""
        };

        return format!("{from}{to}{promo_pt}");
    }

    /// Convert move into UciMove struct of the v_uci crate
    pub fn to_uci(&self) -> v_uci::UciMove {
        let from = self.from().to_uci_square();
        let to = self.to().to_uci_square();

        let promotion = if self.is_promotion() {
            Some(
                self.promotion_piecetype()
                    .expect(".is_promotion check")
                    .to_uci(),
            )
        } else {
            None
        };

        return v_uci::UciMove {
            from,
            to,
            promotion,
        };
    }
}

impl std::cmp::PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
