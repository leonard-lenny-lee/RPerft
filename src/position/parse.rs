/// Contains methods for parsing FEN strings into position representation
/// and serialize the position into other formats.
use super::*;
use constants::bb;
use types::PieceType;

impl Position {
    /// Parse a FEN striing into a position representation
    pub fn from_fen(fen: &str) -> Result<Self, RuntimeError> {
        let tokens: Vec<&str> = fen.trim().split(" ").collect();

        /* FEN strings contain 6 tokens representing
            1. Board
            2. Side to move
            3. Castling rights
            4. En passant target square
            5. Halfmove clock
            6. Fullmove clock
        */
        if tokens.len() != 6 {
            return Err(RuntimeError::ParseFenError);
        }

        // Fill BBSet for white and black. Set 'us' as white for now
        let mut us = BBSet::default();
        let mut them = BBSet::default();
        let mut board_tokens: Vec<&str> = tokens[0].split("/").collect();

        if board_tokens.len() != 8 {
            return Err(RuntimeError::ParseFenError);
        }

        // Reverse vector so index 0 is at square A1
        board_tokens.reverse();
        let board = board_tokens.join("");
        let mut sq = 0;

        for c in board.chars() {
            if sq >= 64 {
                return Err(RuntimeError::ParseFenError);
            }
            let mask = BitBoard::from_square(sq);

            // Alphabetic characters represent a piece of the square
            if c.is_alphabetic() {
                let bbset = if c.is_uppercase() { &mut us } else { &mut them };
                bbset.all |= mask;
                match c {
                    'p' | 'P' => bbset.pawn |= mask,
                    'r' | 'R' => bbset.rook |= mask,
                    'n' | 'N' => bbset.knight |= mask,
                    'b' | 'B' => bbset.bishop |= mask,
                    'q' | 'Q' => bbset.queen |= mask,
                    'k' | 'K' => bbset.king |= mask,
                    _ => return Err(RuntimeError::ParseFenError),
                }
                sq += 1;
            }
            // Numeric pieces represent empty squares
            else if c.is_numeric() {
                let n_empty = c.to_digit(10).expect("carried out .is_numeric check");
                sq += n_empty as usize;
            }
            // Non-alphanumeric characters are invalid
            else {
                return Err(RuntimeError::ParseFenError);
            }
        }

        // All 64 squares must be accounted for
        if sq != 64 {
            return Err(RuntimeError::ParseFenError);
        }

        let occupied = us.all | them.all;
        let free = !occupied;

        // Set side to move
        let (white_to_move, side_to_move) = match tokens[1] {
            "w" => (true, Color::White),
            "b" => (false, Color::Black),
            _ => return Err(RuntimeError::ParseFenError),
        };

        // Set castling rights
        let mut castling_rights = bb::EMPTY;
        for c in tokens[2].chars() {
            match c {
                'K' => castling_rights |= bb::H1,
                'k' => castling_rights |= bb::H8,
                'Q' => castling_rights |= bb::A1,
                'q' => castling_rights |= bb::A8,
                '-' => (),
                _ => return Err(RuntimeError::ParseFenError),
            }
        }

        // Set en passant target square
        let en_passant = if tokens[3] == "-" {
            bb::EMPTY
        } else {
            match BitBoard::from_algebraic(tokens[3]) {
                Ok(bb) => bb,
                Err(_) => return Err(RuntimeError::ParseFenError),
            }
        };

        // Set halfmove clock
        let halfmove_clock = match tokens[4].parse::<u8>() {
            Ok(val) => val,
            Err(_) => return Err(RuntimeError::ParseFenError),
        };

        // Set fullmove clock
        let fullmove_clock = match tokens[5].parse::<u8>() {
            Ok(val) => val,
            Err(_) => return Err(RuntimeError::ParseFenError),
        };

        // Swap us/them pointers if black to move
        if let Color::Black = side_to_move {
            std::mem::swap(&mut us, &mut them)
        }

        let mut pos = Self {
            us,
            them,
            occupied,
            free,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_clock,
            key: 0,
            white_to_move,
            side_to_move,
            ply: 0,
            stack: Vec::new(),
            nnue_pos: NNUEPosition::init(board, side_to_move),
        };

        // Initialize Zobrist key
        pos.key = pos.generate_zobrist_key();
        // Check that the king cannot be captured
        pos.check_legal()?;
        return Ok(pos);
    }

    /// Initialize a new starting position
    pub fn new_starting_position() -> Self {
        return Self::from_fen(constants::fen::START).expect("start fen is valid");
    }

    pub fn copy(&self) -> Self {
        Self {
            us: self.us,
            them: self.them,
            occupied: self.occupied,
            free: self.free,
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            fullmove_clock: self.fullmove_clock,
            key: self.key,
            white_to_move: self.white_to_move,
            side_to_move: self.side_to_move,
            ply: self.ply,
            stack: Vec::new(),
            nnue_pos: self.nnue_pos,
        }
    }

    /// Convert position into a 8 x 8 array of characters
    fn to_array(&self) -> [[char; 8]; 8] {
        let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
        let (white, black) = self.white_black_bitboards();
        let w_array = white.as_array();
        let b_array = black.as_array();

        let (w_charset, b_charset) = (
            [' ', 'P', 'R', 'N', 'B', 'Q', 'K'],
            [' ', 'p', 'r', 'n', 'b', 'q', 'k'],
        );

        for (i, (bb_1, bb_2)) in std::iter::zip(w_array, b_array).enumerate() {
            for (bb, charset) in std::iter::zip([bb_1, bb_2], [w_charset, b_charset]) {
                for sq in bb.forward_scan() {
                    let index = sq.to_square();
                    let (x, y) = (index / 8, index % 8);
                    array[x][y] = charset[i];
                }
            }
        }

        return array;
    }

    /// Generate the FEN string of the position
    pub fn to_fen(&self) -> String {
        let mut tokens = Vec::new();
        let mut array = self.to_array();
        // Reverse for FEN parsing
        array.reverse();

        // Build the board token
        let mut board_token = String::new();
        for row in array {
            let mut n_empty = 0;
            for c in row {
                if c == ' ' {
                    n_empty += 1;
                    continue;
                }
                if n_empty > 0 {
                    board_token.push_str(&n_empty.to_string()[..]);
                    n_empty = 0;
                }
                board_token.push(c)
            }
            // Flush empty
            if n_empty > 0 {
                board_token.push_str(&n_empty.to_string()[..])
            }
            board_token.push('/')
        }
        // Remove the '/' for the final row
        board_token.pop();
        tokens.push(board_token);

        // Parse side to move
        match self.side_to_move {
            Color::White => tokens.push("w".to_string()),
            Color::Black => tokens.push("b".to_string()),
        }

        // Build castling token
        let mut castling_token = String::new();
        for (target_sq, c) in std::iter::zip([bb::H1, bb::A1, bb::H8, bb::A8], ['K', 'Q', 'k', 'q'])
        {
            if self.castling_rights & target_sq != bb::EMPTY {
                castling_token.push(c)
            }
        }
        if castling_token.len() == 0 {
            castling_token.push('-')
        }
        tokens.push(castling_token);

        // Push en passant token
        if self.en_passant != bb::EMPTY {
            tokens.push(self.en_passant.to_algebraic());
        } else {
            tokens.push("-".to_string())
        }

        // Push clock tokens
        tokens.push(self.halfmove_clock.to_string());
        tokens.push(self.fullmove_clock.to_string());

        return tokens.join(" ");
    }

    /// Convert to string representation to visually display the board
    pub fn to_board(&self) -> String {
        let array = self.to_array();
        let mut out = String::new();
        out.push_str("  +---+---+---+---+---+---+---+---+\n8 ");

        for idx in 0..8 {
            let x = 7 - idx;
            let row = array[x];
            if idx != 0 {
                let rank = &(8 - idx).to_string()[..];
                out.push_str("|\n  +---+---+---+---+---+---+---+---+\n");
                out.push_str(rank);
                out.push(' ');
            }
            for c in row {
                out.push_str("| ");
                out.push(c);
                out.push(' ')
            }
        }
        out.push_str("|\n  +---+---+---+---+---+---+---+---+\n");
        out.push_str("    a   b   c   d   e   f   g   h \n");
        return out;
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}\nFen: {}\nKey: {:X}\n",
            self.to_board(),
            self.to_fen(),
            self.key
        )
    }
}

impl BBSet {
    pub fn as_array(&self) -> [&BitBoard; 7] {
        return [
            &self.all,
            &self.pawn,
            &self.rook,
            &self.knight,
            &self.bishop,
            &self.queen,
            &self.king,
        ];
    }
}

impl NNUEPosition {
    pub fn pieces(&self) -> *const usize {
        return self.pieces.as_ptr();
    }

    pub fn squares(&self) -> *const usize {
        return self.squares.as_ptr();
    }

    fn init(token: String, stm: Color) -> NNUEPosition {
        const PIECE_NAME: &str = "_KQRBNPkqrbnp_";
        const NUMBERS: &str = "12345678";

        let mut pieces = [0; 32];
        let mut squares = [64; 32];
        let mut board = [32; 64];

        let mut sq = 0;
        let mut index = 2;

        for c in token.chars() {
            if let Some(pc) = PIECE_NAME.find(c) {
                if pc == 1 {
                    pieces[0] = pc;
                    squares[0] = sq;
                    board[sq] = 0;
                } else if pc == 7 {
                    pieces[1] = pc;
                    squares[1] = sq;
                    board[sq] = 1;
                } else {
                    pieces[index] = pc;
                    squares[index] = sq;
                    board[sq] = index;
                    index += 1;
                }
                sq += 1;
            } else if let Some(_) = NUMBERS.find(c) {
                sq += c.to_digit(10).expect("is number") as usize;
            } else {
                panic!("fen check error")
            }
        }

        return NNUEPosition {
            player: stm as usize,
            pieces,
            squares,
            board,
            end_ptr: index - 1,
        };
    }
}

impl std::ops::Index<PieceType> for BBSet {
    type Output = BitBoard;

    fn index(&self, index: PieceType) -> &Self::Output {
        match index {
            PieceType::Any => &self.all,
            PieceType::Pawn => &self.pawn,
            PieceType::Rook => &self.rook,
            PieceType::Knight => &self.knight,
            PieceType::Bishop => &self.bishop,
            PieceType::Queen => &self.queen,
            PieceType::King => &self.king,
        }
    }
}

impl std::ops::IndexMut<PieceType> for BBSet {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        match index {
            PieceType::Any => &mut self.all,
            PieceType::Pawn => &mut self.pawn,
            PieceType::Rook => &mut self.rook,
            PieceType::Knight => &mut self.knight,
            PieceType::Bishop => &mut self.bishop,
            PieceType::Queen => &mut self.queen,
            PieceType::King => &mut self.king,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::rank::*;

    #[test]
    fn test_new_starting_position() {
        let pos = Position::new_starting_position();

        // White pieces
        assert_eq!(pos.us.all, RANK_1 | RANK_2, "w.any");
        assert_eq!(pos.us.pawn, RANK_2, "w.pawn");
        assert_eq!(pos.us.rook, bb::A1 | bb::H1, "w.rook");
        assert_eq!(pos.us.knight, bb::B1 | bb::G1, "w.knight");
        assert_eq!(pos.us.bishop, bb::C1 | bb::F1, "w.bishop");
        assert_eq!(pos.us.queen, bb::D1, "w.queen");
        assert_eq!(pos.us.king, bb::E1, "w.king");

        // Black pieces
        assert_eq!(pos.them.all, RANK_7 | RANK_8, "b.any");
        assert_eq!(pos.them.pawn, RANK_7, "b.pawn");
        assert_eq!(pos.them.rook, bb::A8 | bb::H8, "b.rook");
        assert_eq!(pos.them.knight, bb::B8 | bb::G8, "b.knight");
        assert_eq!(pos.them.bishop, bb::C8 | bb::F8, "b.bishop");
        assert_eq!(pos.them.queen, bb::D8, "b.queen");
        assert_eq!(pos.them.king, bb::E8, "b.king");

        // Shared bitboards
        let expected_occ = RANK_1 | RANK_2 | RANK_7 | RANK_8;
        let expected_free = !expected_occ;
        assert_eq!(pos.occupied, expected_occ, "occ");
        assert_eq!(pos.free, expected_free, "free");

        // Other token parsing
        assert!(matches!(pos.side_to_move, Color::White));
        assert_eq!(pos.castling_rights, bb::A1 | bb::H1 | bb::A8 | bb::H8);
        assert_eq!(pos.en_passant, bb::EMPTY);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_clock, 1);
    }

    #[test]
    fn test_to_fen() {
        let pos = Position::from_fen(constants::fen::TEST_3).unwrap();
        assert_eq!(pos.to_fen(), constants::fen::TEST_3)
    }

    #[test]
    fn test_nnue_pos_init() {
        use bb::*;
        use nnue::Pieces::*;

        let pos = Position::new_starting_position();

        #[rustfmt::skip]
        let expected_pieces = [
            WKing, BKing,
            WRook, WKnight, WBishop, WQueen, WBishop, WKnight, WRook,
            WPawn, WPawn, WPawn, WPawn, WPawn, WPawn, WPawn, WPawn,
            BPawn, BPawn, BPawn, BPawn, BPawn, BPawn, BPawn, BPawn,
            BRook, BKnight, BBishop, BQueen, BBishop, BKnight, BRook,
        ].map(|x| x as usize);

        #[rustfmt::skip]
        let expected_squares = [
            E1, E8,
            A1, B1, C1, D1, F1, G1, H1,
            A2, B2, C2, D2, E2, F2, G2, H2,
            A7, B7, C7, D7, E7, F7, G7, H7,
            A8, B8, C8, D8, F8, G8, H8,
        ].map(|x| x.to_square());

        #[rustfmt::skip]
        let expected_board = [
             2,  3,  4,  5,  0,  6,  7,  8,
             9, 10, 11, 12, 13, 14, 15, 16,
            32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32,
            17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28,  1, 29, 30, 31,
        ];

        let expected_end_ptr = 31;

        let expected_player = if pos.white_to_move {
            nnue::Colors::White as usize
        } else {
            nnue::Colors::Black as usize
        };

        assert_eq!(expected_pieces, pos.nnue_pos.pieces, "piece failure");
        assert_eq!(expected_squares, pos.nnue_pos.squares, "square failure");
        assert_eq!(expected_board, pos.nnue_pos.board, "board failure");
        assert_eq!(expected_end_ptr, pos.nnue_pos.end_ptr, "pointer failure");
        assert_eq!(expected_player, pos.nnue_pos.player, "player failutre")
    }

    #[ignore]
    #[test]
    // Run manually and inspect
    fn test_to_board() {
        let pos = Position::from_fen(constants::fen::TEST_3).unwrap();
        print!("{}", pos.to_board())
    }
}
