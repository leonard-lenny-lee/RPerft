use crate::evaluator;
use crate::mechanics::bittools;
use crate::mechanics::Piece;
use crate::mechanics::PromotionPiece;
use crate::mechanics::PawnMove;
use crate::mechanics::JumpingPiece;
use crate::mechanics::SlidingPiece;
use crate::mechanics::SpecialMove;
use crate::mechanics::ASCIIBases;
use crate::mechanics::Maps;

const DEFAULT_FEN: &str= "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Declare rank masks
pub const RANK_1: u64 = 0x00000000000000ff;
pub const RANK_2: u64 = 0x000000000000ff00;
pub const RANK_3: u64 = 0x0000000000ff0000;
pub const RANK_4: u64 = 0x00000000ff000000;
pub const RANK_5: u64 = 0x000000ff00000000;
pub const RANK_6: u64 = 0x0000ff0000000000;
pub const RANK_7: u64 = 0x00ff000000000000;
pub const RANK_8: u64 = 0xff00000000000000;

// Declare file masks
pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

// Castle masks [KingsideMask, KingsideTarget, QueensideMask, QueensideTarget]
pub const W_CASTLE: [u64; 4] = [0x60, 0x40, 0xe, 0x4];
pub const B_CASTLE: [u64; 4] = [0x6000000000000000, 0x4000000000000000, 0xe00000000000000, 0x400000000000000];

struct Move {
    target: u64,
    src: u64,
    moved_piece: Piece,
    promotion_piece: PromotionPiece,
    special_move_flag: SpecialMove,
    is_capture: bool,
    captured_piece: Piece,
}

impl Move {
    fn new(target_sq: u64, src_sq: u64, moved_piece: &Piece, 
        promotion_piece: PromotionPiece, special_move_flag: SpecialMove, 
        position: &Position) -> Move {
            let o_pieces;
            if position.white_to_move {
                o_pieces = &position.b_pieces;
            } else {
                o_pieces = &position.w_pieces;
            }
            let is_capture = o_pieces[0] & target_sq != 0;
            let mut captured_piece = Piece::Any;
            if is_capture {
                for piece in Piece::iterator() {
                    if o_pieces[piece as usize] & target_sq != 0 {
                        // Identified which piece has been captured
                        captured_piece = piece;
                        break;
                    }
                }
            }
            return Move {
                target: target_sq,
                src: src_sq,
                moved_piece: *moved_piece,
                promotion_piece: promotion_piece,
                special_move_flag: special_move_flag,
                is_capture: is_capture,
                captured_piece: captured_piece,
            };
        }

}
pub struct Position {
    w_pieces: [u64; 7],
    b_pieces: [u64; 7],
    occ: u64,
    free: u64,
    white_to_move: bool,
    w_kingside_castle: bool,
    b_kingside_castle: bool,
    w_queenside_castle: bool,
    b_queenside_castle: bool,
    en_passant_target_sq: u64,
    halfmove_clock: i8,
    fullmove_clock: i8,
}

impl Position {

    pub fn new_from_fen(fen: Option<String>) -> Position {
        let fen = fen.unwrap_or(String::from(DEFAULT_FEN));
        let split_fen: Vec<&str> = fen.split(" ").collect();
        assert!(split_fen.len() == 6);
        let board = split_fen[0];
        // Initialise bitboard
        let mut w_pieces: [u64; 7] = [0; 7];
        let mut b_pieces: [u64; 7] = [0; 7];
        // Fill bitboards
        let split_board = board.split("/");
        for (y, rank) in split_board.enumerate() {
            for (x, piece) in rank.chars().enumerate() {
                if piece.is_alphabetic() {
                    let mask: u64 = 1 << ((7 - y) * 8 + x);
                    match piece {
                        'P' => w_pieces[Piece::Pawn as usize] |= mask,
                        'p' => b_pieces[Piece::Pawn as usize] |= mask,
                        'R' => w_pieces[Piece::Rook as usize] |= mask,
                        'r' => b_pieces[Piece::Rook as usize] |= mask,
                        'N' => w_pieces[Piece::Knight as usize] |= mask,
                        'n' => b_pieces[Piece::Knight as usize] |= mask,
                        'B' => w_pieces[Piece::Bishop as usize] |= mask,
                        'b' => b_pieces[Piece::Bishop as usize] |= mask,
                        'Q' => w_pieces[Piece::Queen as usize] |= mask,
                        'q' => b_pieces[Piece::Queen as usize] |= mask,
                        'K' => w_pieces[Piece::King as usize] |= mask,
                        'k' => b_pieces[Piece::King as usize] |= mask,
                        _ => (),
                    }
                    if piece.is_uppercase() {
                        w_pieces[Piece::Any as usize] |= mask
                    } else {
                        b_pieces[Piece::Any as usize] |= mask
                    }
                } else {
                    continue
                }
            }
        }
        let occ = w_pieces[0] | b_pieces[0];
        let free = !occ;
        // Populate other fields
        let white_to_move: bool = split_fen[1] == "w";
        let mut w_kingside_castle: bool = false;
        let mut b_kingside_castle: bool = false;
        let mut w_queenside_castle: bool = false;
        let mut b_queenside_castle: bool = false;
        for c in split_fen[2].chars() {
            match c {
                'K' => w_kingside_castle = true,
                'Q' => w_queenside_castle = true,
                'k' => b_kingside_castle = true,
                'q' => b_queenside_castle = true,
                _ => (),
            }
        };
        // Calculate en passant target square
        let mut en_passant_target_sq: u64 = 0;
        let epts: Vec<char> = split_fen[3].chars().collect();
        if epts[0] != '-' {
            assert!(epts.len() == 2);
            let file = epts[0] as u8;
            let rank = epts[1] as u8;
            en_passant_target_sq = 1 << ((file - ASCIIBases::LowerA as u8)
                + (rank - ASCIIBases::Zero as u8) * 8);
        }
        // Calculate clocks
        let halfmove_clock: i8 = split_fen[4].parse().unwrap();
        let fullmove_clock: i8 = split_fen[5].parse().unwrap();
        // Construct struct with calculated values
        return Position {
            w_pieces, b_pieces, occ, free, white_to_move, w_kingside_castle,
            b_kingside_castle, w_queenside_castle, b_queenside_castle,
            en_passant_target_sq, halfmove_clock, fullmove_clock
        }
    }

    // Methods to generate the target maps for pawn moves

    fn get_wpawn_sgl_pushes(&self) -> u64 {
        self.w_pieces[1] << 8 & self.free
    }
    
    fn get_wpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.w_pieces[1] & RANK_2) << 8 & self.free;
        sgl_push << 8 & self.free
    }
    
    fn get_wpawn_left_captures(&self) -> u64 {
        (self.w_pieces[1] ^ FILE_A) << 7 & self.b_pieces[0]
    }
    
    fn get_wpawn_right_captures(&self) -> u64 {
        (self.w_pieces[1] ^ FILE_H) << 9 & self.b_pieces[0]
    }

    fn get_wpawn_left_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pieces[1] ^ FILE_A) << 7 & self.en_passant_target_sq
    }

    fn get_wpawn_right_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pieces[1] ^ FILE_H) << 9 & self.en_passant_target_sq
    }
    
    fn get_bpawn_sgl_pushes(&self) -> u64 {
        self.b_pieces[1] >> 8 & self.free
    }
    
    fn get_bpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.b_pieces[1] & RANK_7) >> 8 & self.free;
        sgl_push >> 8 & self.free
    }
    
    fn get_bpawn_left_captures(&self) -> u64 {
        (self.b_pieces[1] ^ FILE_A) >> 9 & self.w_pieces[0]
    }

    fn get_bpawn_right_captures(&self) -> u64 {
        (self.b_pieces[1] ^ FILE_H) >> 7 & self.w_pieces[0]
    }
    
    fn get_bpawn_left_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pieces[1] ^ FILE_A) >> 9 & self.en_passant_target_sq
    }

    fn get_bpawn_right_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pieces[1] ^ FILE_H) >> 7 & self.en_passant_target_sq
    }

    // Move generation functions

    fn generate_pawn_moves(&self, move_type: PawnMove) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let targets: u64;
        let srcs: u64;
        let promotion_rank: u64;
        if self.white_to_move {
            match move_type {
                PawnMove::SinglePush => {
                    targets = self.get_wpawn_sgl_pushes();
                    srcs = targets >> 8;
                },
                PawnMove::DoublePush => {
                    targets = self.get_wpawn_dbl_pushes();
                    srcs = targets >> 16;
                },
                PawnMove::CaptureLeft => {
                    targets = self.get_wpawn_left_captures();
                    srcs = targets >> 7;
                },
                PawnMove::CaptureRight => {
                    targets = self.get_wpawn_right_captures();
                    srcs = targets >> 9;
                }
            }
            promotion_rank = RANK_8;
        } else {
            match move_type {
                PawnMove::SinglePush => {
                    targets = self.get_bpawn_sgl_pushes();
                    srcs = targets << 8;
                },
                PawnMove::DoublePush => {
                    targets = self.get_bpawn_dbl_pushes();
                    srcs = targets << 16;
                },
                PawnMove::CaptureLeft => {
                    targets = self.get_bpawn_left_captures();
                    srcs = targets << 9;
                }
                PawnMove::CaptureRight => {
                    targets = self.get_bpawn_right_captures();
                    srcs = targets << 7;
                }
            }
            promotion_rank = RANK_1;
        }
        let target_vec = bittools::forward_scan(targets);
        let src_vec = bittools::forward_scan(srcs);
        for i in 0..target_vec.len() {
            let src = src_vec[i];
            let target = target_vec[i];
            if target & promotion_rank == 0 {
                moves.push(
                    Move::new(
                        target,
                        src, 
                        &Piece::Pawn,
                        PromotionPiece::None, 
                        SpecialMove::None,
                        self)
                )
            } else {
                let mut promotions = self.generate_promotions(target, src);
                moves.append(&mut promotions);
            }
        }
        return moves;
    }

    fn generate_jumping_moves(
        &self, piece: JumpingPiece, f_pieces: &[u64; 7], maps: &Maps
    ) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let srcs;
        let map;
        let moved_piece;
        match piece {
            JumpingPiece::Knight => {
                srcs = f_pieces[Piece::Knight as usize];
                map = &maps.knight;
                moved_piece = Piece::Knight;
            },
            JumpingPiece::King => {
                srcs = f_pieces[Piece::King as usize];
                map = &maps.king;
                moved_piece = Piece::King;
            }
        }
        let src_vec = bittools::forward_scan(srcs);
        for src in src_vec {
            let targets = map[bittools::ilsb(&src)] ^ f_pieces[Piece::Any as usize];
            let target_vec = bittools::forward_scan(targets);
            for target in target_vec {
                moves.push(
                    Move::new(
                        target,
                        src,
                        &moved_piece,
                        PromotionPiece::None,
                        SpecialMove::None,
                        self,
                    )
                )
            }
        }
        return moves;
    }

    fn generate_sliding_moves(
        &self, piece: SlidingPiece, f_pieces: &[u64; 7], maps: &Maps
    ) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let srcs: u64;
        let masks: Vec<&[u64; 64]>;
        let moved_piece;
        match piece {
            SlidingPiece::Bishop => {
                srcs = f_pieces[Piece::Bishop as usize];
                masks = vec![&maps.diag, &maps.adiag];
                moved_piece = Piece::Bishop;
            },
            SlidingPiece::Rook => {
                srcs = f_pieces[Piece::Rook as usize];
                masks = vec![&maps.file, &maps.rank];
                moved_piece = Piece::Rook;
            },
            SlidingPiece::Queen => {
                srcs = f_pieces[Piece::Queen as usize];
                masks = vec![&maps.diag, &maps.adiag, &maps.file, &maps.rank];
                moved_piece = Piece::Queen;
            }
        }
        let src_vec = bittools::forward_scan(srcs);
        for src in src_vec {
            let mut targets: u64 = 0;
            for mask in &masks {
                targets |= search_engine::hyp_quint(self.occ, src, mask);
            }
            targets ^= f_pieces[Piece::Any as usize];
            let target_vec = bittools::forward_scan(targets);
            for target in target_vec {
                moves.push(
                    Move::new(
                        target,
                        src,
                        &moved_piece,
                        PromotionPiece::None,
                        SpecialMove::None,
                        self,
                    )
                )
            }
        }
        return moves;
    }

    // Special Moves

    fn generate_promotions(&self, target: u64, src: u64) -> Vec<Move> {
        let mut promotions: Vec<Move> = Vec::new();
        for piece in PromotionPiece::iterator() {
            promotions.push(
                Move::new(
                    target,
                    src,
                    &Piece::Pawn,
                    piece,
                    SpecialMove::Promotion,
                    self,                    
                )
            )
        }
        return promotions;
    }

    fn generate_en_passant_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let target = self.en_passant_target_sq;
        if target == 0 {
            return moves;
        }
        let mut src_vec: Vec<u64> = Vec::new();
        if self.white_to_move {
            src_vec.push(self.get_wpawn_left_en_passant() >> 7);
            src_vec.push(self.get_wpawn_right_en_passant() >> 9);
        } else {
            src_vec.push(self.get_bpawn_left_en_passant() << 9);
            src_vec.push(self.get_bpawn_right_en_passant() << 7);
        }
        for src in src_vec {
            if src != 0 {
                moves.push(
                    Move::new(
                        target,
                        src,
                        &Piece::Pawn,
                        PromotionPiece::None,
                        SpecialMove::EnPassant,
                        self,
                    )
                )
            }
        }
        return moves;
    }

    fn generate_castling_moves(&self, m: &[u64; 4], r: &[bool; 2], f_pieces: &[u64; 7]) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let src = f_pieces[Piece::King as usize];
        for i in 0..2 {
            if r[i] && m[i*2] & self.occ == 0 {
                moves.push(
                    Move::new(
                        m[i*2+1],
                        src,
                        &Piece::King,
                        PromotionPiece::None,
                        SpecialMove::Castling,
                        self,
                    )
                )
            }
        }
        return moves;
    }

    pub fn generate_moves(&self, maps: &Maps) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let f_pieces: &[u64; 7];
        let o_pieces: &[u64; 7];
        let castle_masks: &[u64; 4];
        let castle_rights;
        if self.white_to_move {
            f_pieces = &self.w_pieces;
            o_pieces = &self.b_pieces;
            castle_masks = &W_CASTLE;
            castle_rights = [self.w_kingside_castle, self.w_queenside_castle];
        } else {
            f_pieces = &self.b_pieces;
            o_pieces = &self.w_pieces;
            castle_masks = &B_CASTLE;
            castle_rights = [self.b_kingside_castle, self.b_kingside_castle];
        }
        // Pawn single pushes
        moves.append(&mut self.generate_pawn_moves(PawnMove::SinglePush));
        // Pawn double pushes
        moves.append(&mut self.generate_pawn_moves(PawnMove::DoublePush));
        // Pawn left captures
        moves.append(&mut self.generate_pawn_moves(PawnMove::CaptureLeft));
        // Pawn right captures
        moves.append(&mut self.generate_pawn_moves(PawnMove::CaptureRight));
        // Knight moves
        moves.append(&mut self.generate_jumping_moves(JumpingPiece::Knight, f_pieces, maps));
        // King moves
        moves.append(&mut self.generate_jumping_moves(JumpingPiece::King, f_pieces, maps));
        // Bishop moves
        moves.append(&mut self.generate_sliding_moves(SlidingPiece::Bishop, f_pieces, maps));
        // Rook moves
        moves.append(&mut self.generate_sliding_moves(SlidingPiece::Rook, f_pieces, maps));
        // Queen moves
        moves.append(&mut self.generate_sliding_moves(SlidingPiece::Queen, f_pieces, maps));
        // Castling
        moves.append(&mut self.generate_castling_moves(castle_masks, &castle_rights, f_pieces));

        if self.en_passant_target_sq != 0 {
            moves.append(&mut self.generate_en_passant_moves());
        }

        return moves;
    }

    pub fn make_move(&mut self, mv: &Move) {
        let f_pieces;
        let o_pieces;

        if self.white_to_move {
            f_pieces = &mut self.w_pieces;
            o_pieces = &mut self.b_pieces;
        } else {
            f_pieces = &mut self.b_pieces;
            o_pieces = &mut self.w_pieces;
        }
        // Free up src squares and occupy target squares
        self.occ ^= mv.src;
        self.free |= mv.src;
        f_pieces[0] ^= mv.src | mv.target;
        f_pieces[mv.moved_piece as usize] ^= mv.src | mv.target;
        self.occ |= mv.target;
        if mv.is_capture {
            o_pieces[mv.captured_piece as usize] ^= mv.target;
            o_pieces[0] ^= mv.target;
        }
        // Set en passant target sq if the move was a double pawn push
        if matches!(mv.moved_piece, Piece::Pawn) 
            && (((mv.src << 16) == mv.target) | ((mv.src >> 16) == mv.target)) {
                if self.white_to_move {
                    self.en_passant_target_sq = mv.src << 8;
                } else {
                    self.en_passant_target_sq = mv.src >> 8;
                }
        } else {
            self.en_passant_target_sq = 0;
        }
        // Set the clocks
        if mv.is_capture || matches!(mv.moved_piece, Piece::Pawn) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if !self.white_to_move {
            self.fullmove_clock += 1;
        }
    }

    pub fn unmake_move(&self, mv: &Move) {

    }
    
}

pub struct Score {
    value: i64,
}

impl Score {
    pub fn new() -> Score {
        let val: i64 = 0;
        return Score {value: val};
    }
}
pub struct GameContext {
    position: Position,
    score: Score,
}

// This initializes the game context
impl GameContext {

    pub fn new_from_fen(fen: Option<String>) -> GameContext {
        let position = Position::new_from_fen(fen);
        let score = evaluator::evaluate(position);
        return GameContext {position, score};
    }
}