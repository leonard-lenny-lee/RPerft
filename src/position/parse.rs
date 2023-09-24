/// Contains methods for parsing FEN strings into position representation
/// and serialize the position into other formats.
use super::*;

use std::iter::zip;

use constants::bb;
use types::PieceT;

impl Position {
    /// Parse a FEN string into a position representation
    pub fn from_fen(fen: &str) -> Result<Self, ()> {
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
            return Err(());
        }

        // Fill BBSet for white and black. Set 'us' as white for now
        let mut us = BitBoardSet::default();
        let mut them = BitBoardSet::default();
        let mut board_tokens: Vec<&str> = tokens[0].split("/").collect();

        if board_tokens.len() != 8 {
            return Err(());
        }

        // Reverse vector so index 0 is at square A1
        board_tokens.reverse();
        let board = board_tokens.join("");
        let mut sq = 0;

        for c in board.chars() {
            if sq >= 64 {
                return Err(());
            }
            let mask = BitBoard::from_sq(sq);

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
                    _ => return Err(()),
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
                return Err(());
            }
        }

        // All 64 squares must be accounted for
        if sq != 64 {
            return Err(());
        }

        let occ = us.all | them.all;
        let free = !occ;

        // Set side to move
        let (wtm, stm) = match tokens[1] {
            "w" => (true, ColorT::White),
            "b" => (false, ColorT::Black),
            _ => return Err(()),
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
                _ => return Err(()),
            }
        }

        // Set en passant target square
        let ep_sq = if tokens[3] == "-" {
            bb::EMPTY
        } else {
            match BitBoard::from_algebraic(tokens[3]) {
                Ok(bb) => bb,
                Err(_) => return Err(()),
            }
        };

        // Set halfmove clock
        let halfmove_clock = match tokens[4].parse::<u8>() {
            Ok(val) => val,
            Err(_) => return Err(()),
        };

        // Set fullmove clock
        let fullmove_clock = match tokens[5].parse::<u8>() {
            Ok(val) => val,
            Err(_) => return Err(()),
        };

        // Swap us/them pointers if black to move
        if let ColorT::Black = stm {
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
        };

        // Initialize Zobrist key
        pos.key = pos.generate_zobrist_key();
        // Check that the king cannot be captured
        pos.check_legal()?;
        return Ok(pos);
    }

    /// Initialize a new starting position
    pub fn new_start_pos() -> Self {
        return Self::from_fen(constants::fen::STARTING_FEN).expect("start fen is valid");
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

        for (i, (bb_1, bb_2)) in zip(w_array, b_array).enumerate() {
            for (bb, charset) in zip([bb_1, bb_2], [w_charset, b_charset]) {
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
            ColorT::White => tokens.push("w".to_string()),
            ColorT::Black => tokens.push("b".to_string()),
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
        if self.ep_sq != bb::EMPTY {
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

impl BitBoardSet {
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

impl std::ops::Index<PieceT> for BitBoardSet {
    type Output = BitBoard;

    fn index(&self, index: PieceT) -> &Self::Output {
        match index {
            PieceT::Any => &self.all,
            PieceT::Pawn => &self.pawn,
            PieceT::Rook => &self.rook,
            PieceT::Knight => &self.knight,
            PieceT::Bishop => &self.bishop,
            PieceT::Queen => &self.queen,
            PieceT::King => &self.king,
        }
    }
}

impl std::ops::IndexMut<PieceT> for BitBoardSet {
    fn index_mut(&mut self, index: PieceT) -> &mut Self::Output {
        match index {
            PieceT::Any => &mut self.all,
            PieceT::Pawn => &mut self.pawn,
            PieceT::Rook => &mut self.rook,
            PieceT::Knight => &mut self.knight,
            PieceT::Bishop => &mut self.bishop,
            PieceT::Queen => &mut self.queen,
            PieceT::King => &mut self.king,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::rank::*;

    #[test]
    fn test_start_pos() {
        let pos = Position::new_start_pos();

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
        assert_eq!(pos.occ, expected_occ, "occ");
        assert_eq!(pos.free, expected_free, "free");

        // Other token parsing
        assert!(matches!(pos.stm, ColorT::White));
        assert_eq!(pos.castling_rights, bb::A1 | bb::H1 | bb::A8 | bb::H8);
        assert_eq!(pos.ep_sq, bb::EMPTY);
        assert_eq!(pos.halfmove_clock, 0);
        assert_eq!(pos.fullmove_clock, 1);
    }

    #[test]
    fn test_to_fen() {
        let pos = Position::from_fen(constants::fen::TEST_3).unwrap();
        assert_eq!(pos.to_fen(), constants::fen::TEST_3)
    }

    #[ignore]
    #[test]
    // Run manually and inspect
    fn test_to_board() {
        let pos = Position::from_fen(constants::fen::TEST_3).unwrap();
        print!("{}", pos.to_board())
    }
}
