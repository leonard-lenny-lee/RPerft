
const DEFAULT_FEN: &str= "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Declare rank masks
const RANK_1: u64 = 0x00000000000000ff;
const RANK_2: u64 = 0x000000000000ff00;
const RANK_3: u64 = 0x0000000000ff0000;
const RANK_4: u64 = 0x00000000ff000000;
const RANK_5: u64 = 0x000000ff00000000;
const RANK_6: u64 = 0x0000ff0000000000;
const RANK_7: u64 = 0x00ff000000000000;
const RANK_8: u64 = 0xff00000000000000;

// Declare file masks
const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_C: u64 = 0x0404040404040404;
const FILE_D: u64 = 0x0808080808080808;
const FILE_E: u64 = 0x1010101010101010;
const FILE_F: u64 = 0x2020202020202020;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

// Declare castling masks
const W_KS_CASTLE: u64 = 0x60;
const W_KS_C_TARG: u64 = 0x40;
const W_QS_CASTLE: u64 = 0xe;
const W_QS_C_TARG: u64 = 0x4;
const B_KS_CASTLE: u64 = 0x6000000000000000;
const B_KS_C_TARG: u64 = 0x4000000000000000;
const B_QS_CASTLE: u64 = 0xe00000000000000;
const B_QS_C_TARG: u64 = 0x400000000000000;

// Functions to precompute lookup maps
pub fn generate_knight_maps() -> Vec<u64> {
    let mut maps: Vec<u64> = Vec::new();
    for i in 0..64 {
        let mut map: u64 = 0;
        let origin: u64 = 1 << i;
        map |= (origin ^ FILE_H) << 17; // NNE
        map |= (origin ^ (FILE_G | FILE_H)) << 10; // NEE
        map |= (origin ^ (FILE_G | FILE_H)) >> 6; // SEE
        map |= (origin ^ FILE_H) >> 15; // SSE
        map |= (origin ^ FILE_A) >> 17; // SSW
        map |= (origin ^ (FILE_A | FILE_B)) >> 10;// SWW
        map |= (origin ^ (FILE_A | FILE_B)) << 6; // NWW
        map |= (origin ^ FILE_A) << 15; // NNW
        maps.push(map);
    }
    return maps;
}

pub fn generate_king_maps() -> Vec<u64> {
    let mut maps: Vec<u64> = Vec::new();
    for i in 0..64 {
        let mut map: u64 = 0;
        let origin: u64 = 1 << i;
        map |= origin << 8; // N
        map |= (origin ^ FILE_H) << 9; // NE
        map |= (origin ^ FILE_H) << 1; // E
        map |= (origin ^ FILE_H) >> 7; // SE
        map |= origin >> 8; // S
        map |= (origin ^ FILE_A) >> 9; // SW
        map |= (origin ^ FILE_A) >> 1; // W
        map |= (origin ^ FILE_A) << 7; // NW
        maps.push(map);
    }
    return maps;
}

pub fn generate_rank_masks() -> Vec<u64> {
    let mut masks: Vec<u64> = Vec::new();
    for i in 0..64 {
        match i / 8 {
            0 => masks.push(RANK_1),
            1 => masks.push(RANK_2),
            2 => masks.push(RANK_3),
            3 => masks.push(RANK_4),
            4 => masks.push(RANK_5),
            5 => masks.push(RANK_6),
            6 => masks.push(RANK_7),
            7 => masks.push(RANK_8),
            _ => (),
        }
    }
    return masks;
}

pub fn generate_file_masks() -> Vec<u64> {
    let mut masks: Vec<u64> = Vec::new();
    for i in 0..64 {
        match i % 8 {
            0 => masks.push(FILE_A),
            1 => masks.push(FILE_B),
            2 => masks.push(FILE_C),
            3 => masks.push(FILE_D),
            4 => masks.push(FILE_E),
            5 => masks.push(FILE_F),
            6 => masks.push(FILE_G),
            7 => masks.push(FILE_H),
            _ => (),
        }
    }
    return masks;
}

pub fn generate_diagonal_masks() -> Vec<u64> {
    let mut masks: Vec<u64> = Vec::new();
    for i in 0..64 {
        let mut mask: u64 = 1 << i;
        let from_left = i % 8;
        let from_right = 7 - from_left;
        for l in 1..from_left+1 {
            let l_trans = i + l * -9;
            if l_trans >= 0 {
                mask |= 1 << (l_trans);
            } else {
                break;
            }
        }
        for r in 1..from_right+1 {
            let r_trans = i + r * 9;
            if r_trans < 64 {
                mask |= 1 << (r_trans);
            } else {
                break;
            }
        }
        masks.push(mask)
    }
    return masks;
}

pub fn generate_antidiagonal_masks() -> Vec<u64> {
    let mut masks: Vec<u64> = Vec::new();
    for i in 0..64 {
        let mut mask: u64 = 1 << i;
        let from_left = i % 8;
        let from_right = 7 - from_left;
        for l in 1..from_left+1 {
            let l_trans = i + l * 7;
            if l_trans < 64 {
                mask |= 1 << (l_trans);
            } else {
                break;
            }
        }
        for r in 1..from_right+1 {
            let r_trans = i + r * -7;
            if r_trans >= 0 {
                mask |= 1 << (r_trans);
            } else {
                break;
            }
        }
        masks.push(mask)
    }
    return masks;
}

pub struct LookupTables {
    pub rank_masks: Vec<u64>,
    pub file_masks: Vec<u64>,
    pub diagonal_masks: Vec<u64>,
    pub antidiagonal_masks: Vec<u64>,
    pub knight_maps: Vec<u64>,
    pub king_maps: Vec<u64>,
}

pub fn hyp_quint(o: u64, s: u64, m: &Vec<u64>) -> u64 {
    let m = m[s.trailing_zeros() as usize];
    let mut forward: u64 = o & m;
    let mut reverse: u64 = forward.reverse_bits();
    forward = forward.wrapping_sub(2 * s);
    reverse = reverse.wrapping_sub(2 * s.reverse_bits());
    forward ^= reverse.reverse_bits();
    forward &= m;
    return forward;
}

pub enum ASCIIBases {
    LowerA = 97,
    UpperA = 65,
    Zero = 48,
}

pub enum PromotionPiece {
    Rook = 0,
    Knight = 1,
    Bishop = 2,
    Queen = 3,
}

pub enum SpecialMove {
    None = 0,
    Promotion = 1,
    EnPassant = 2,
    Castling = 3,
}

pub enum PawnMove {
    SinglePush,
    DoublePush,
    CaptureLeft,
    CaptureRight,
}

pub enum SlidingPiece {
    Bishop,
    Rook,
}

// Bitscan functions
pub fn get_lsb(n: &u64) -> u64 {
    1 << n.trailing_zeros()
}

pub fn forward_scan(mut n: u64) -> Vec<u64> {
    let mut scan_result: Vec<u64> = Vec::new();
    while n != 0 {
        let lsb = get_lsb(&n);
        scan_result.push(lsb);
        n ^= lsb;
    }
    scan_result
}

pub fn new_move(target_sq: u64, src_sq: u64, promotion_piece: PromotionPiece,
    special_move_flag: SpecialMove) -> u16 {
        let mut mv: u16 = 0;
        mv |= target_sq.trailing_zeros() as u16;
        mv |= (src_sq.trailing_zeros() as u16) << 6;
        mv |= (promotion_piece as u16) << 12;
        mv |= (special_move_flag as u16) << 14;
        return mv
    }

pub fn position_from_fen(fen: String) -> Position {
    let split_fen: Vec<&str> = fen.split(" ").collect();
    assert!(split_fen.len() == 6);
    let board = split_fen[0];
    // Initialise bitboard
    let mut w_pawn: u64 = 0;
    let mut b_pawn: u64 = 0;
    let mut w_rook: u64 = 0;
    let mut b_rook: u64 = 0;
    let mut w_knight: u64 = 0;
    let mut b_knight: u64 = 0;
    let mut w_bishop: u64 = 0;
    let mut b_bishop: u64 = 0;
    let mut w_queen: u64 = 0;
    let mut b_queen: u64 = 0;
    let mut w_king: u64 = 0;
    let mut b_king: u64 = 0;
    let mut w_piece: u64 = 0;
    let mut b_piece: u64 = 0;
    // Fill bitboards
    let split_board = board.split("/");
    for (y, rank) in split_board.enumerate() {
        for (x, piece) in rank.chars().enumerate() {
            if piece.is_alphabetic() {
                let mask: u64 = 1 << ((7 - y) * 8 + x);
                match piece {
                    'P' => w_pawn |= mask,
                    'p' => b_pawn |= mask,
                    'R' => w_rook |= mask,
                    'r' => b_rook |= mask,
                    'N' => w_knight |= mask,
                    'n' => b_knight |= mask,
                    'B' => w_bishop |= mask,
                    'b' => b_bishop |= mask,
                    'Q' => w_queen |= mask,
                    'q' => b_queen |= mask,
                    'K' => w_king |= mask,
                    'k' => b_king |= mask,
                    _ => (),
                }
                if piece.is_uppercase() {
                    w_piece |= mask
                } else {
                    b_piece |= mask
                }
            } else {
                continue
            }
        }
    }
    let occ = w_piece | b_piece;
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
        w_pawn, b_pawn, w_rook, b_rook, w_knight, b_knight, w_bishop, b_bishop,
        w_queen, b_queen, w_king, b_king, w_piece, b_piece, occ, free,
        white_to_move, w_kingside_castle, b_kingside_castle, w_queenside_castle,
        b_queenside_castle, en_passant_target_sq, halfmove_clock, fullmove_clock
    }
}

pub struct Position {
    w_pawn: u64,
    b_pawn: u64,
    w_rook: u64,
    b_rook: u64,
    w_knight: u64,
    b_knight: u64,
    w_bishop: u64,
    b_bishop: u64,
    w_queen: u64,
    b_queen: u64,
    w_king: u64,
    b_king: u64,
    w_piece: u64,
    b_piece: u64,
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

    // Methods to generate the target maps for pawn moves

    fn get_wpawn_sgl_pushes(&self) -> u64 {
        self.w_pawn << 8 & self.free
    }
    
    fn get_wpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.w_pawn & RANK_2) << 8 & self.free;
        sgl_push << 8 & self.free
    }
    
    fn get_wpawn_left_captures(&self) -> u64 {
        (self.w_pawn ^ FILE_A) << 7 & self.b_piece
    }
    
    fn get_wpawn_right_captures(&self) -> u64 {
        (self.w_pawn ^ FILE_H) << 9 & self.b_piece
    }

    fn get_wpawn_left_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pawn ^ FILE_A) << 7 & self.en_passant_target_sq
    }

    fn get_wpawn_right_en_passant(&self) -> u64 {
        assert!(self.white_to_move);
        (self.w_pawn ^ FILE_H) << 9 & self.en_passant_target_sq
    }
    
    fn get_bpawn_sgl_pushes(&self) -> u64 {
        self.b_pawn >> 8 & self.free
    }
    
    fn get_bpawn_dbl_pushes(&self) -> u64 {
        let sgl_push: u64 = (self.b_pawn & RANK_7) >> 8 & self.free;
        sgl_push >> 8 & self.free
    }
    
    fn get_bpawn_left_captures(&self) -> u64 {
        (self.b_pawn ^ FILE_A) >> 9 & self.w_piece
    }

    fn get_bpawn_right_captures(&self) -> u64 {
        (self.b_pawn ^ FILE_H) >> 7 & self.w_piece
    }
    
    fn get_bpawn_left_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pawn ^ FILE_A) >> 9 & self.en_passant_target_sq
    }

    fn get_bpawn_right_en_passant(&self) -> u64 {
        assert!(!self.white_to_move);
        (self.b_pawn ^ FILE_H) >> 7 & self.en_passant_target_sq
    }

    // Move generation functions

    fn generate_pawn_moves(&self, move_type: PawnMove) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
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
        let target_vec = forward_scan(targets);
        let src_vec = forward_scan(srcs);
        for i in 0..target_vec.len() {
            let src = src_vec[i];
            let target = target_vec[i];
            if target & promotion_rank == 0 {
                moves.push(
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::None
                    )
                )
            } else {
                let mut promotions = self.generate_promotions(target, src);
                moves.append(&mut promotions);
            }
        }
        return moves;
    }

    fn generate_knight_moves(&self, map: &Vec<u64>) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let srcs: u64;
        let f_pieces: u64;
        if self.white_to_move {
            srcs = self.w_knight;
            f_pieces = self.w_piece;
        } else {
            srcs = self.b_knight;
            f_pieces = self.b_piece;
        }
        let src_vec = forward_scan(srcs);
        for src in src_vec {
            let targets = map[src.trailing_zeros() as usize] ^ f_pieces;
            let target_vec = forward_scan(targets);
            for target in target_vec {
                moves.push(
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::None
                    )
                )
            }
        }
        return moves;
    }

    fn generate_king_moves(&self, map: &Vec<u64>) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let src: u64;
        let f_pieces: u64;
        if self.white_to_move {
            src = self.w_king;
            f_pieces = self.w_piece;
        } else {
            src = self.b_king;
            f_pieces = self.b_piece;
        }
        let targets = map[src.trailing_ones() as usize] ^ f_pieces;
        let target_vec = forward_scan(targets);
        for target in target_vec {
            moves.push(
                new_move(
                    target,
                    src,
                    PromotionPiece::Rook,
                    SpecialMove::None,
                )
            )
        }
        return moves;
    }

    fn generate_sliding_moves(&self, piece: SlidingPiece, t: &LookupTables) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let srcs: u64;
        let f_pieces: u64;
        if self.white_to_move {
            match piece {
                SlidingPiece::Bishop => srcs = self.w_bishop,
                SlidingPiece::Rook => srcs = self.w_rook,
            }
            f_pieces = self.w_piece;
        } else {
            match piece {
                SlidingPiece::Bishop => srcs = self.b_bishop,
                SlidingPiece::Rook => srcs = self.b_rook,
            }
            f_pieces = self.b_piece;
        }
        let m_1: &Vec<u64>;
        let m_2: &Vec<u64>;
        match piece {
            SlidingPiece::Bishop => {
                m_1 = &t.diagonal_masks;
                m_2 = &t.antidiagonal_masks;
            },
            SlidingPiece::Rook => {
                m_1 = &t.file_masks;
                m_2 = &t.rank_masks;
            }
        }

        let src_vec = forward_scan(srcs);
        for src in src_vec {
            let targets_1 = hyp_quint(self.occ, src, m_1);
            let targets_2 = hyp_quint(self.occ, src, m_2);
            let targets =  (targets_1 | targets_2) ^ f_pieces;
            let target_vec = forward_scan(targets);
            for target in target_vec {
                moves.push(
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::None
                    )
                )
            }
        }
        return moves;
    }

    fn generate_rook_moves(&self, m_h: &Vec<u64>, m_v:&Vec<u64>) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let srcs: u64;
        let f_pieces: u64;
        if self.white_to_move {
            srcs = self.w_rook;
            f_pieces = self.w_piece;
        } else {
            srcs = self.b_rook;
            f_pieces = self.b_piece;
        }
        let src_vec = forward_scan(srcs);
        for src in src_vec {
            let h_targets = hyp_quint(self.occ, src, m_h);
            let v_targets = hyp_quint(self.occ, src, m_v);
            let targets =  (v_targets | h_targets) ^ f_pieces;
            let target_vec = forward_scan(targets);
            for target in target_vec {
                moves.push(
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::None
                    )
                )
            }
        }
        return moves;
    }

    fn generate_bishop_moves(&self, m_d: &Vec<u64>, m_a:&Vec<u64>) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let srcs: u64;
        let f_pieces: u64;
        if self.white_to_move {
            srcs = self.w_bishop;
            f_pieces = self.w_piece;
        } else {
            srcs = self.b_bishop;
            f_pieces = self.b_piece;
        }
        let src_vec = forward_scan(srcs);
        for src in src_vec {
            let d_targets = hyp_quint(self.occ, src, m_d);
            let a_targets = hyp_quint(self.occ, src, m_a);
            let targets =  (d_targets | a_targets) ^ f_pieces;
            let target_vec = forward_scan(targets);
            for target in target_vec {
                moves.push(
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::None
                    )
                )
            }
        }
        return moves;
    }

    fn generate_queen_moves(&self, t: &LookupTables) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let src: u64;
        let f_pieces: u64;
        if self.white_to_move {
            src = self.w_queen;
            f_pieces = self.w_piece;
        } else {
            src = self.b_queen;
            f_pieces = self.b_piece;
        }
        let d_targets = hyp_quint(self.occ, src, &t.diagonal_masks);
        let a_targets = hyp_quint(self.occ, src, &t.antidiagonal_masks);
        let h_targets = hyp_quint(self.occ, src, &t.rank_masks);
        let v_targets = hyp_quint(self.occ, src, &t.file_masks);
        let targets =  (d_targets | a_targets | v_targets | h_targets) ^ f_pieces;
        let target_vec = forward_scan(targets);
        for target in target_vec {
            moves.push(
                new_move(
                    target,
                    src,
                    PromotionPiece::Rook,
                    SpecialMove::None
                )
            )
        }
        return moves;
    }

    // Special Moves

    fn generate_promotions(&self, target: u64, src: u64) -> Vec<u16> {
        let mut promotions: Vec<u16> = Vec::new();
        let promotion_pieces = [
            PromotionPiece::Rook, PromotionPiece::Bishop,
            PromotionPiece::Knight, PromotionPiece::Queen
        ];
        for piece in promotion_pieces{
            promotions.push(
                new_move(
                    target,
                    src,
                    piece,
                    SpecialMove::Promotion
                )
            )
        }
        return promotions;
    }

    fn generate_en_passant_moves(&self) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
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
                    new_move(
                        target,
                        src,
                        PromotionPiece::Rook,
                        SpecialMove::EnPassant
                    )
                )
            }
        }
        return moves;
    }

    fn generate_castling_moves(&self) -> Vec<u16> {
        let mut moves: Vec<u16> = Vec::new();
        let ksc: bool; let qsc: bool;
        let ksm: u64; let qsm: u64;
        let kst: u64; let qst: u64;
        let src: u64;
        if self.white_to_move {
            ksc = self.w_kingside_castle;
            qsc = self.w_queenside_castle;
            ksm = W_KS_CASTLE;
            qsm = W_QS_CASTLE;
            kst = W_KS_C_TARG;
            qst = W_QS_C_TARG;
            src = self.w_king;
        } else {
            ksc = self.b_kingside_castle;
            qsc = self.b_queenside_castle;
            ksm = B_KS_CASTLE;
            qsm = B_QS_CASTLE;
            kst = B_KS_C_TARG;
            qst = B_QS_C_TARG;
            src = self.b_king;
        }
        if ksc && ksm & self.occ == 0 {
            moves.push(
                new_move(
                    kst,
                    src,
                    PromotionPiece::Rook,
                    SpecialMove::Castling,
                )
            )
        }
        if qsc && qsm & self.occ == 0 {
            moves.push(
                new_move(
                    qst,
                    src,
                    PromotionPiece::Rook,
                    SpecialMove::Castling
                )
            )
        }
        return moves;
    }
    
}

pub fn search_positions(position: Position, depth: i32) {

}