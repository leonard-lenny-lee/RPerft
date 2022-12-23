/// Contains the methods required to parse a FEN string into a Data struct
/// The data struct holds all the data required to describe a position.

use super::*;

#[derive(Clone, Copy)]
pub struct Data {
    pub w_pieces: PieceSet,
    pub b_pieces: PieceSet,
    pub occ: BB,
    pub free: BB,
    pub white_to_move: bool,
    pub castling_rights: BB,
    pub en_passant_target_sq: BB,
    pub halfmove_clock: i8,
    pub fullmove_clock: i8,
}

impl Data {

    // Methods required to parse a FEN string into a Data struct
    
    pub fn from_fen(fen: String) -> Data {
        let tokens: Vec<&str> = fen.trim().split(" ").collect();
        assert!(tokens.len() == 6);
        let mut pos = Data::new();
        pos.init_bitboards(tokens[0]);
        pos.init_white_to_move(tokens[1]);
        pos.init_castling_rights(tokens[2]);
        pos.init_en_passant(tokens[3]);
        pos.init_halfmove_clock(tokens[4]);
        pos.init_fullmove_clock(tokens[5]);
        pos
    }
    
    pub fn new() -> Data {
        Data {
            w_pieces: PieceSet::new(),
            b_pieces: PieceSet::new(),
            occ: EMPTY_BB,
            free: EMPTY_BB,
            white_to_move: true,
            castling_rights: EMPTY_BB,
            en_passant_target_sq: EMPTY_BB,
            halfmove_clock: 0,
            fullmove_clock: 0
        }
    }

    /// Initialise a set of bitboards for white and black pieces from the 
    /// portion of the FEN string representing the board. Also sets the master
    /// occupied and free bitboards 
    fn init_bitboards(&mut self, board: &str) {
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
            let mask = BB::from_index(i as usize);
            if char.is_alphabetic() {
                // If the character is alphabetic, then it represents a piece;
                // populate the relevant bitboard
                let pieceinit_to_modify;
                if char.is_uppercase() {
                    pieceinit_to_modify = &mut w_pieces;
                } else {
                    pieceinit_to_modify = &mut b_pieces;
                    char.make_ascii_uppercase();
                }
                pieceinit_to_modify.any |= mask;
                match char {
                    'P' => pieceinit_to_modify.pawn |= mask,
                    'R' => pieceinit_to_modify.rook |= mask,
                    'N' => pieceinit_to_modify.knight |= mask,
                    'B' => pieceinit_to_modify.bishop |= mask,
                    'Q' => pieceinit_to_modify.queen |= mask,
                    'K' => pieceinit_to_modify.king |= mask,
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
    fn init_white_to_move(&mut self, code: &str) {
        assert!(code == "w" || code == "b");
        self.white_to_move = code == "w";
    }

    /// Set the castling rights of a position
    fn init_castling_rights(&mut self, code: &str) {
        if code.contains("K") {
            self.castling_rights |= W_KINGSIDE_ROOK_STARTING_SQ
        }
        if code.contains("k") {
            self.castling_rights |= B_KINGSIDE_ROOK_STARTING_SQ
        }
        if code.contains("Q") {
            self.castling_rights |= W_QUEENSIDE_ROOK_STARTING_SQ
        }
        if code.contains("q") {
            self.castling_rights |= B_QUEENSIDE_ROOK_STARTING_SQ
        }
    }

    /// Calculate the en passant target square bitmask
    fn init_en_passant(&mut self, epts: &str) {
        let target_sq;
        if epts == "-" {
            target_sq = EMPTY_BB;
        } else {
            target_sq = BB::from_algebraic(epts);
        }
        self.en_passant_target_sq = target_sq;
    }

    /// Set the halfmove clock
    fn init_halfmove_clock(&mut self, clock: &str) {
        let halfmove_clock: i8;
        match clock.parse() {
            Ok(c) => halfmove_clock = c,
            Err(_e) => panic!("Invalid halfmove clock")
        }
        self.halfmove_clock = halfmove_clock;
    }

    /// Set the fullmove clock
    fn init_fullmove_clock(&mut self, clock: &str) {
        let fullmove_clock: i8;
        match clock.parse() {
            Ok(c) => fullmove_clock = c,
            Err(_e) => panic!("Invalid fullmove clock")
        }
        self.fullmove_clock = fullmove_clock;
    }

    /// The difference between the number of queens on the board
    pub fn queen_diff(&self) -> i32 {
        self.w_pieces.n_queens() - self.b_pieces.n_queens()
    }

    /// The difference between the number of rooks on the board
    pub fn rook_diff(&self) -> i32 {
        self.w_pieces.n_rooks() - self.b_pieces.n_rooks()
    }

    /// The difference between the number of bishops on the board
    pub fn bishop_diff(&self) -> i32 {
        self.w_pieces.n_bishops() - self.b_pieces.n_bishops()
    }

    /// The difference between the number of knights on the board
    pub fn knight_diff(&self) -> i32 {
        self.w_pieces.n_knights() - self.b_pieces.n_knights()
    }

    /// The difference between the number of pawns on the board
    pub fn pawn_diff(&self) -> i32 {
        self.w_pieces.n_pawns() - self.b_pieces.n_pawns()
    }

    /// The total number of queens on the board
    pub fn queen_sum(&self) -> i32 {
        self.w_pieces.n_queens() + self.b_pieces.n_queens()
    }

    /// The total number of rooks on the board
    pub fn rook_sum(&self) -> i32 {
        self.w_pieces.n_rooks() + self.b_pieces.n_rooks()
    }

    /// The total number of bishops on the board
    pub fn bishop_sum(&self) -> i32 {
        self.w_pieces.n_bishops() + self.b_pieces.n_bishops()
    }

    /// The total number of knights on the board
    pub fn knight_sum(&self) -> i32 {
        self.w_pieces.n_knights() + self.b_pieces.n_knights()
    }

    /// The total number of pawns on the board
    pub fn pawn_sum(&self) -> i32 {
        self.w_pieces.n_pawns() + self.b_pieces.n_pawns()
    }

    fn to_array(&self, pretty: bool) -> [[char; 8]; 8] {
        let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
        let w_array = self.w_pieces.as_array();
        let b_array = self.b_pieces.as_array();
        let (w_char_set, b_char_set) = if pretty {
            ([' ', '\u{2659}', '\u{2656}', '\u{2658}', '\u{2657}', '\u{2655}', '\u{2654}'],
             [' ', '\u{265f}', '\u{265c}', '\u{265e}', '\u{265d}', '\u{265b}', '\u{265a}'])
        } else {
            ([' ', 'P', 'R', 'N', 'B', 'Q', 'K'], [' ', 'p', 'r', 'n', 'b', 'q', 'k'])
        };
        for i in 1..7 {

            for bit in b_array[i].forward_scan() {
                let index = bit.to_index();
                let x = index / 8;
                let y = index % 8;
                array[x][y] = b_char_set[i];
            }

            for bit in w_array[i].forward_scan() {
                let index = bit.to_index();
                let x = index / 8;
                let y = index % 8;
                array[x][y] = w_char_set[i];
            }
        };
        array
    }

    pub fn fen(&self) -> String {

        let array = self.to_array(false);
        let mut out = String::new();
        for i in 0..8 {
            let i2 = 7 - i;
            let row = array[i2];
            let mut n_empty = 0;
            for c in row {
                if c != ' ' {
                    if n_empty > 0 {
                        out.push_str(&n_empty.to_string()[..]);
                        n_empty = 0
                    }
                    out.push(c);
                } else {
                    n_empty += 1;
                }
            }
            if n_empty > 0 {
                out.push_str(&n_empty.to_string()[..])
            }
            out.push('/')
        }
    
        if self.white_to_move {
            out.push_str(" w ")
        } else {
            out.push_str(" b ")
        }
        
        if self.castling_rights & W_KINGSIDE_ROOK_STARTING_SQ != EMPTY_BB {
            out.push('K')
        }
        if self.castling_rights & W_QUEENSIDE_ROOK_STARTING_SQ != EMPTY_BB {
            out.push('Q')
        }
        if self.castling_rights & B_KINGSIDE_ROOK_STARTING_SQ != EMPTY_BB {
            out.push('k')
        }
        if self.castling_rights & B_QUEENSIDE_ROOK_STARTING_SQ != EMPTY_BB {
            out.push('q')
        }

        if self.en_passant_target_sq != EMPTY_BB {
            out.push(' ');
            out.push_str(&self.en_passant_target_sq.to_algebraic()[..]);
        } else {
            out.push_str(" -")
        }
        out.push(' ');
        out.push_str(&self.halfmove_clock.to_string()[..]);
        out.push(' ');
        out.push_str(&self.fullmove_clock.to_string()[..]);
        out
    }

    /// Convert to string representation of the board for printing to the
    /// standard output
    pub fn board(&self) -> String {

        let array = self.to_array(true);
        let mut out = String::new();
        out.push_str("   --- --- --- --- --- --- --- --- \n8 ");
        for i in 0..8 {
            let i2 = 7 - i;
            let row = array[i2];
            if i != 0 {
                let rank = &(8 - i).to_string()[..];
                out.push_str("|\n   --- --- --- --- --- --- --- --- \n");
                out.push_str(rank);
                out.push(' ');
            }
            for c in row {
                out.push_str("| ");
                out.push(c);
                out.push(' ')
            }
        }
        out.push_str(
            "|\n   --- --- --- --- --- --- --- --- \n    a   b   c   d   e   f   g   h \n"
        );
        return out
    }

    /// Convert the board into a string for display
    pub fn to_string(&self) -> String {
        let mut out = self.board();
        out.push_str("\nFEN: ");
        out.push_str(&self.fen()[..]);
        out.push_str("\n\n");
        out
    }

}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_init_bitboards() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let mut data = Data::new();
        data.init_bitboards(fen);
        // White pieces
        assert_eq!(data.w_pieces.any, RANK_1 | RANK_2, "w.any");
        assert_eq!(data.w_pieces.pawn, RANK_2, "w.pawn");
        assert_eq!(data.w_pieces.rook, BB::from_indices(vec![0, 7]), "w.rook");
        assert_eq!(data.w_pieces.knight, BB::from_indices(vec![1, 6]), "w.knight");
        assert_eq!(data.w_pieces.bishop, BB::from_indices(vec![2, 5]), "w.bishop");
        assert_eq!(data.w_pieces.queen, BB::from_indices(vec![3]), "w.queen");
        assert_eq!(data.w_pieces.king, BB::from_indices(vec![4]), "w.king");
        // Black pieces
        assert_eq!(data.b_pieces.any, RANK_7 | RANK_8, "b.any");
        assert_eq!(data.b_pieces.pawn, RANK_7, "b.pawn");
        assert_eq!(data.b_pieces.rook, BB::from_indices(vec![56, 63]), "b.rook");
        assert_eq!(data.b_pieces.knight, BB::from_indices(vec![57, 62]), "b.knight");
        assert_eq!(data.b_pieces.bishop, BB::from_indices(vec![58, 61]), "b.bishop");
        assert_eq!(data.b_pieces.queen, BB::from_indices(vec![59]), "b.queen");
        assert_eq!(data.b_pieces.king, BB::from_indices(vec![60]), "b.king");
        // Universal bitboards
        let expected_occ = RANK_1 | RANK_2 | RANK_7 | RANK_8;
        let expected_free = !expected_occ;
        assert_eq!(data.occ, expected_occ, "occ");
        assert_eq!(data.free, expected_free, "free");

    }

    #[test_case("w", true; "white")]
    #[test_case("b", false; "black")]
    fn test_init_white_to_move (test_case: &str, expected: bool) {
        let mut data = Data::new();
        data.init_white_to_move(test_case);
        assert_eq!(data.white_to_move, expected)
    }

    #[test]
    #[should_panic]
    fn test_invalid_white_to_move() {
        let mut data = Data::new();
        data.init_white_to_move("X")
    }

    #[test]
    fn test_init_castling_rights() {
        let mut data = Data::new();
        data.init_castling_rights("KkQq");
        assert_eq!(
            W_KINGSIDE_ROOK_STARTING_SQ | B_KINGSIDE_ROOK_STARTING_SQ |
            W_QUEENSIDE_ROOK_STARTING_SQ | B_QUEENSIDE_ROOK_STARTING_SQ,
            data.castling_rights
        )
    }

    #[test_case("-", EMPTY_BB; "n_empty")]
    #[test_case("e6", BB::from_index(44); "e6")]
    fn test_init_en_passant(test: &str, expected: BB) {
        let mut data = Data::new();
        data.init_en_passant(test);
        assert_eq!(data.en_passant_target_sq, expected)
    }

    #[test]
    fn test_init_halfmove_clock() {
        let mut data = Data::new();
        data.init_halfmove_clock("6");
        assert_eq!(data.halfmove_clock, 6)
    }

    #[test]
    fn test_init_fullmove_clock() {
        let mut data = Data::new();
        data.init_fullmove_clock("0");
        assert_eq!(data.fullmove_clock, 0)
    }

    #[test]
    #[ignore]
    fn test_fen_parse() {
        let data = Data::from_fen(DEFAULT_FEN.to_string());
        print!("{}", data.to_string())
    }

}