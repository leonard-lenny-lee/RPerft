/// Contains the methods required to parse a FEN string into a Data struct
/// The data struct holds all the data required to describe a position.

use super::*;

impl Data {
    
    pub fn from_fen(fen: String) -> Data {
        let split_fen: Vec<&str> = fen.split(" ").collect();
        assert!(split_fen.len() == 6);
        let mut pos = Data::new();
        pos.set_bitboards(split_fen[0]);
        pos.set_white_to_move(split_fen[1]);
        pos.set_castling_rights(split_fen[2]);
        pos.set_en_passant(split_fen[3]);
        pos.set_halfmove_clock(split_fen[4]);
        pos.set_fullmove_clock(split_fen[5]);
        pos
    }
    
    fn new() -> Data {
        Data {
            w_pieces: PieceSet::new(),
            b_pieces: PieceSet::new(),
            occ: EMPTY_BB,
            free: EMPTY_BB,
            white_to_move: true,
            w_kingside_castle: false,
            b_kingside_castle: false,
            w_queenside_castle: false,
            b_queenside_castle: false,
            en_passant_target_sq: EMPTY_BB,
            halfmove_clock: 0,
            fullmove_clock: 0
        }
    }

    /// Initialise a set of bitboards for white and black pieces from the 
    /// portion of the FEN string representing the board. Also sets the master
    /// occupied and free bitboards 
    fn set_bitboards(&mut self, board: &str) {
        let mut w_pieces: PieceSet = PieceSet::new();
        let mut b_pieces: PieceSet = PieceSet::new();
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
                let pieceset_to_modify;
                if char.is_uppercase() {
                    pieceset_to_modify = &mut w_pieces;
                } else {
                    pieceset_to_modify = &mut b_pieces;
                    char.make_ascii_uppercase();
                }
                pieceset_to_modify.any |= mask;
                match char {
                    'P' => pieceset_to_modify.pawn |= mask,
                    'R' => pieceset_to_modify.rook |= mask,
                    'N' => pieceset_to_modify.knight |= mask,
                    'B' => pieceset_to_modify.bishop |= mask,
                    'Q' => pieceset_to_modify.queen |= mask,
                    'K' => pieceset_to_modify.king |= mask,
                    _ => panic!("Invalid character {} in FEN", char)
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
        self.w_pieces = w_pieces;
        self.b_pieces = b_pieces;
        self.occ = w_pieces.any | b_pieces.any;
        self.free = !self.occ;
    }

    /// Set white to move field
    fn set_white_to_move(&mut self, code: &str) {
        assert!(code == "w" || code == "b");
        self.white_to_move = code == "w";
    }

    /// Set the castling rights of a position
    fn set_castling_rights(&mut self, code: &str) {
        self.w_kingside_castle = code.contains("K");
        self.b_kingside_castle = code.contains("k");
        self.w_queenside_castle = code.contains("Q");
        self.b_queenside_castle = code.contains("q");
    }

    /// Calculate the en passant target square bitmask
    fn set_en_passant(&mut self, epts: &str) {
        let target_sq;
        if epts == "-" {
            target_sq = EMPTY_BB;
        } else {
            target_sq = bittools::algebraic_to_bitmask(epts);
        }
        self.en_passant_target_sq = target_sq;
    }

    /// Set the halfmove clock
    fn set_halfmove_clock(&mut self, clock: &str) {
        let halfmove_clock: i8;
        match clock.parse() {
            Ok(c) => halfmove_clock = c,
            Err(_e) => panic!("Invalid halfmove clock")
        }
        self.halfmove_clock = halfmove_clock;
    }

    /// Set the fullmove clock
    fn set_fullmove_clock(&mut self, clock: &str) {
        let fullmove_clock: i8;
        match clock.parse() {
            Ok(c) => fullmove_clock = c,
            Err(_e) => panic!("Invalid fullmove clock")
        }
        self.fullmove_clock = fullmove_clock;
    }

}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use super::*;
    use bittools::squares_to_bitboard as stb;
    use test_case::test_case;

    #[test]
    fn test_from_fen_init() {
        Data::from_fen(DEFAULT_FEN.to_string());
    }

    #[test]
    fn test_new_init() {
        Data::new();
    }

    #[test]
    fn test_set_bitboards() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let mut data = Data::new();
        data.set_bitboards(fen);
        // White pieces
        assert_eq!(data.w_pieces.any, RANK_1 | RANK_2, "w.any");
        assert_eq!(data.w_pieces.pawn, RANK_2, "w.pawn");
        assert_eq!(data.w_pieces.rook, stb(vec![0, 7]), "w.rook");
        assert_eq!(data.w_pieces.knight, stb(vec![1, 6]), "w.knight");
        assert_eq!(data.w_pieces.bishop, stb(vec![2, 5]), "w.bishop");
        assert_eq!(data.w_pieces.queen, stb(vec![3]), "w.queen");
        assert_eq!(data.w_pieces.king, stb(vec![4]), "w.king");
        // Black pieces
        assert_eq!(data.b_pieces.any, RANK_7 | RANK_8, "b.any");
        assert_eq!(data.b_pieces.pawn, RANK_7, "b.pawn");
        assert_eq!(data.b_pieces.rook, stb(vec![56, 63]), "b.rook");
        assert_eq!(data.b_pieces.knight, stb(vec![57, 62]), "b.knight");
        assert_eq!(data.b_pieces.bishop, stb(vec![58, 61]), "b.bishop");
        assert_eq!(data.b_pieces.queen, stb(vec![59]), "b.queen");
        assert_eq!(data.b_pieces.king, stb(vec![60]), "b.king");
        // Universal bitboards
        let expected_occ = RANK_1 | RANK_2 | RANK_7 | RANK_8;
        let expected_free = !expected_occ;
        assert_eq!(data.occ, expected_occ, "occ");
        assert_eq!(data.free, expected_free, "free");

    }

    #[test_case("w", true; "white")]
    #[test_case("b", false; "black")]
    fn test_set_white_to_move (test_case: &str, expected: bool) {
        let mut data = Data::new();
        data.set_white_to_move(test_case);
        assert_eq!(data.white_to_move, expected)
    }

    #[test]
    #[should_panic]
    fn test_invalid_white_to_move() {
        let mut data = Data::new();
        data.set_white_to_move("X")
    }

    #[test]
    fn test_set_castling_rights() {
        let mut data = Data::new();
        data.set_castling_rights("KkQq");
        assert_eq!(
            data.w_kingside_castle 
            && data.b_kingside_castle
            && data.w_queenside_castle
            && data.b_queenside_castle,
            true
        )
    }

    #[test_case("-", EMPTY_BB; "empty")]
    #[test_case("e6", 1 << 44; "e6")]
    fn test_set_en_passant(test: &str, expected: u64) {
        let mut data = Data::new();
        data.set_en_passant(test);
        assert_eq!(data.en_passant_target_sq, expected)
    }

    #[test]
    fn test_set_halfmove_clock() {
        let mut data = Data::new();
        data.set_halfmove_clock("6");
        assert_eq!(data.halfmove_clock, 6)
    }

    #[test]
    fn test_set_fullmove_clock() {
        let mut data = Data::new();
        data.set_fullmove_clock("0");
        assert_eq!(data.fullmove_clock, 0)
    }
}