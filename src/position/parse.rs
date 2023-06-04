/// Contains methods for parsing FEN strings into position representation
/// and serialize the position into other formats.
use super::*;
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
        let mut us = BBSet::new_empty();
        let mut them = BBSet::new_empty();
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
            let mask = BB::from_sq(sq);

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

        let occ = us.all | them.all;
        let free = !occ;

        // Set side to move
        let (wtm, stm) = match tokens[1] {
            "w" => (true, Color::White),
            "b" => (false, Color::Black),
            _ => return Err(RuntimeError::ParseFenError),
        };

        // Set castling rights
        let mut castling_rights = EMPTY_BB;
        for c in tokens[2].chars() {
            match c {
                'K' => castling_rights |= square::H1,
                'k' => castling_rights |= square::H8,
                'Q' => castling_rights |= square::A1,
                'q' => castling_rights |= square::A8,
                '-' => (),
                _ => return Err(RuntimeError::ParseFenError),
            }
        }

        // Set en passant target square
        let ep_sq = if tokens[3] == "-" {
            EMPTY_BB
        } else {
            match BB::from_algebraic(tokens[3]) {
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
        if let Color::Black = stm {
            std::mem::swap(&mut us, &mut them)
        }

        let mut pos = Self {
            us,
            them,
            occ,
            free,
            castling_rights,
            ep_sq,
            halfmove_clock,
            fullmove_clock,
            key: 0,
            wtm,
            stm,
            ply: 0,
            stack: Vec::new(),
            nnue_pos: NNUEPosition::init(board, stm),
        };

        // Initialize Zobrist key
        pos.key = pos.generate_key();
        // Check that the king cannot be captured
        pos.check_legal()?;
        pos.stack.push(StackData::default());
        return Ok(pos);
    }

    /// Initialize a new starting position
    pub fn new_starting_pos() -> Self {
        return Self::from_fen(STARTPOS).expect("hardcoded starting fen is valid");
    }

    pub fn copy(&self) -> Self {
        Self {
            us: self.us,
            them: self.them,
            occ: self.occ,
            free: self.free,
            castling_rights: self.castling_rights,
            ep_sq: self.ep_sq,
            halfmove_clock: self.halfmove_clock,
            fullmove_clock: self.fullmove_clock,
            key: self.key,
            wtm: self.wtm,
            stm: self.stm,
            ply: self.ply,
            stack: Vec::new(),
            nnue_pos: self.nnue_pos,
        }
    }

    /// Convert position into a 8 x 8 array of characters
    fn to_array(&self) -> [[char; 8]; 8] {
        let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
        let (white, black) = self.white_black();
        let w_array = white.as_array();
        let b_array = black.as_array();

        let (w_charset, b_charset) = (
            [' ', 'P', 'R', 'N', 'B', 'Q', 'K'],
            [' ', 'p', 'r', 'n', 'b', 'q', 'k'],
        );

        for (i, (bb_1, bb_2)) in std::iter::zip(w_array, b_array).enumerate() {
            for (bb, charset) in std::iter::zip([bb_1, bb_2], [w_charset, b_charset]) {
                for sq in bb.forward_scan() {
                    let index = sq.to_sq();
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
        match self.stm {
            Color::White => tokens.push("w".to_string()),
            Color::Black => tokens.push("b".to_string()),
        }

        // Build castling token
        let mut castling_token = String::new();
        for (target_sq, c) in std::iter::zip(
            [square::H1, square::A1, square::H8, square::A8],
            ['K', 'Q', 'k', 'q'],
        ) {
            if self.castling_rights & target_sq != EMPTY_BB {
                castling_token.push(c)
            }
        }
        if castling_token.len() == 0 {
            castling_token.push('-')
        }
        tokens.push(castling_token);

        // Push en passant token
        if self.ep_sq != EMPTY_BB {
            tokens.push(self.ep_sq.to_algebraic());
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
    fn new_empty() -> Self {
        return Self {
            all: EMPTY_BB,
            pawn: EMPTY_BB,
            rook: EMPTY_BB,
            knight: EMPTY_BB,
            bishop: EMPTY_BB,
            queen: EMPTY_BB,
            king: EMPTY_BB,
        };
    }

    pub fn as_array(&self) -> [&BB; 7] {
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
        let mut squares = [0; 32];
        let mut board = [0; 64];

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
                    squares[index] = pc;
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
    type Output = BB;

    fn index(&self, index: PieceType) -> &Self::Output {
        match index {
            PieceType::All => &self.all,
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
            PieceType::All => &mut self.all,
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

    #[test]
    fn test_new_starting_position() {
        let pos = Position::new_starting_pos();

        // White pieces
        assert_eq!(pos.us.all, RANK_1 | RANK_2, "w.any");
        assert_eq!(pos.us.pawn, RANK_2, "w.pawn");
        assert_eq!(pos.us.rook, square::A1 | square::H1, "w.rook");
        assert_eq!(pos.us.knight, square::B1 | square::G1, "w.knight");
        assert_eq!(pos.us.bishop, square::C1 | square::F1, "w.bishop");
        assert_eq!(pos.us.queen, square::D1, "w.queen");
        assert_eq!(pos.us.king, square::E1, "w.king");

        // Black pieces
        assert_eq!(pos.them.all, RANK_7 | RANK_8, "b.any");
        assert_eq!(pos.them.pawn, RANK_7, "b.pawn");
        assert_eq!(pos.them.rook, square::A8 | square::H8, "b.rook");
        assert_eq!(pos.them.knight, square::B8 | square::G8, "b.knight");
        assert_eq!(pos.them.bishop, square::C8 | square::F8, "b.bishop");
        assert_eq!(pos.them.queen, square::D8, "b.queen");
        assert_eq!(pos.them.king, square::E8, "b.king");

        // Shared bitboards
        let expected_occ = RANK_1 | RANK_2 | RANK_7 | RANK_8;
        let expected_free = !expected_occ;
        assert_eq!(pos.occ, expected_occ, "occ");
        assert_eq!(pos.free, expected_free, "free");

        // Other token parsing
        assert!(matches!(pos.stm, Color::White));
        assert_eq!(
            pos.castling_rights,
            square::A1 | square::H1 | square::A8 | square::H8
        );
        assert_eq!(pos.ep_sq, EMPTY_BB);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_clock, 1);
    }

    #[test]
    fn test_to_fen() {
        let pos = Position::from_fen(TPOS3).unwrap();
        assert_eq!(pos.to_fen(), TPOS3)
    }

    #[ignore]
    #[test]
    // Run manually and inspect
    fn test_to_board() {
        let pos = Position::from_fen(TPOS3).unwrap();
        print!("{}", pos.to_board())
    }
}
