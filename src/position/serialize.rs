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

        // Fill BBSet for white and black
        let mut white = BBSet::new_empty();
        let mut black = BBSet::new_empty();
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
            let mask = BB::from_index(sq);

            // Alphabetic characters represent a piece of the square
            if c.is_alphabetic() {
                let bbset = if c.is_uppercase() {
                    &mut white
                } else {
                    &mut black
                };
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

        let occupied_squares = white.all | black.all;
        let free_squares = !occupied_squares;

        // Set side to move
        let side_to_move = match tokens[1] {
            "w" => Color::White,
            "b" => Color::Black,
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
        let en_passant_target_square = if tokens[3] == "-" {
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

        let mut pos = Self {
            white,
            black,
            occupied_squares,
            free_squares,
            castling_rights,
            en_passant_target_square,
            halfmove_clock,
            fullmove_clock,
            key: 0,
            side_to_move,
        };

        // Initialize Zobrist key
        pos.key = pos.generate_key();
        pos.check_legality()?;
        return Ok(pos);
    }

    /// Initialize a new starting position
    pub fn new_starting_pos() -> Self {
        return Self::from_fen(STARTPOS).expect("hardcoded starting fen is valid");
    }

    /// Convert position into a 8 x 8 array of characters
    fn to_array(&self) -> [[char; 8]; 8] {
        let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
        let w_array = self.white.as_array();
        let b_array = self.black.as_array();

        let (w_charset, b_charset) = (
            [' ', 'P', 'R', 'N', 'B', 'Q', 'K'],
            [' ', 'p', 'r', 'n', 'b', 'q', 'k'],
        );

        for i in 1..7 {
            for sq in b_array[i].forward_scan() {
                let index = sq.to_index();
                let (x, y) = (index / 8, index % 8);
                array[x][y] = b_charset[i];
            }

            for sq in w_array[i].forward_scan() {
                let index = sq.to_index();
                let (x, y) = (index / 8, index % 8);
                array[x][y] = w_charset[i];
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
        if self.en_passant_target_square != EMPTY_BB {
            tokens.push(self.en_passant_target_square.to_algebraic());
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

    fn as_array(&self) -> [&BB; 7] {
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
        assert_eq!(pos.white.all, RANK_1 | RANK_2, "w.any");
        assert_eq!(pos.white.pawn, RANK_2, "w.pawn");
        assert_eq!(pos.white.rook, square::A1 | square::H1, "w.rook");
        assert_eq!(pos.white.knight, square::B1 | square::G1, "w.knight");
        assert_eq!(pos.white.bishop, square::C1 | square::F1, "w.bishop");
        assert_eq!(pos.white.queen, square::D1, "w.queen");
        assert_eq!(pos.white.king, square::E1, "w.king");

        // Black pieces
        assert_eq!(pos.black.all, RANK_7 | RANK_8, "b.any");
        assert_eq!(pos.black.pawn, RANK_7, "b.pawn");
        assert_eq!(pos.black.rook, square::A8 | square::H8, "b.rook");
        assert_eq!(pos.black.knight, square::B8 | square::G8, "b.knight");
        assert_eq!(pos.black.bishop, square::C8 | square::F8, "b.bishop");
        assert_eq!(pos.black.queen, square::D8, "b.queen");
        assert_eq!(pos.black.king, square::E8, "b.king");

        // Shared bitboards
        let expected_occ = RANK_1 | RANK_2 | RANK_7 | RANK_8;
        let expected_free = !expected_occ;
        assert_eq!(pos.occupied_squares, expected_occ, "occ");
        assert_eq!(pos.free_squares, expected_free, "free");

        // Other token parsing
        assert!(matches!(pos.side_to_move, Color::White));
        assert_eq!(
            pos.castling_rights,
            square::A1 | square::H1 | square::A8 | square::H8
        );
        assert_eq!(pos.en_passant_target_square, EMPTY_BB);
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
        let pos = Position::from_fen(STARTPOS).unwrap();
        print!("{}", pos.to_board())
    }
}
