/// Contains the methods required to parse a FEN string into a position

use super::*;

impl Position {
    
    pub fn new_from_fen(fen: String) -> Position {
        let split_fen: Vec<&str> = fen.split(" ").collect();
        assert!(split_fen.len() == 6);
        let mut pos = Position::new();
        pos.set_bitboards(split_fen[0]);
        pos.set_white_to_move(split_fen[1]);
        pos.set_castling_rights(split_fen[2]);
        pos.set_en_passant(split_fen[3]);
        pos.set_halfmove_clock(split_fen[4]);
        pos.set_fullmove_clock(split_fen[5]);
        pos
    }
    
    fn new() -> Position {
        Position {
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
            fullmove_clock: 0,
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
        self.halfmove_clock = fullmove_clock;
    }

}