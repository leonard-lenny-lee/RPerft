/// Module containing methods to extract information from a position

use super::*;


impl Position {

    /// Return a bitboard with all squares the opponent pieces are attacking
    pub fn unsafe_squares(&self) -> BB {
        let mut unsafe_squares = EMPTY_BB;
        // Remove our king from the occupancy bitboard for sliding piece move
        // generation to prevent the king from blocking other unsafe squares
        let occ = self.data.occ ^ self.our_pieces().king;
        // Calculate pawn attacks
        unsafe_squares |= self.unsafe_squares_pawn();
        // Calculate attacks in horizontal and vertical directions
        unsafe_squares |= (
            self.their_pieces().rook | self.their_pieces().queen
        ).rook_attacks(occ);
        // Calculate attacks in the diagonal and anti-diagonal directions
        unsafe_squares |= (
            self.their_pieces().bishop | self.their_pieces().queen
        ).bishop_attacks(occ);
        // Calculate knight attacks
        unsafe_squares |= self.unsafe_squares_knight();
        // Calculate king attacks
        unsafe_squares |= self.their_pieces().king.lookup_king_attacks();
        return unsafe_squares
    }

    /// Return all the squares attacked by opponent knights
    pub fn unsafe_squares_knight(&self) -> BB {
        let mut knights = self.their_pieces().knight;
        let mut unsafe_squares = EMPTY_BB;
        while knights.is_any() {
            let knight = knights.pop_ls1b();
            let attacks = knight.lookup_knight_attacks();
            unsafe_squares |= attacks;
        }
        unsafe_squares
    }

    /// Return a bitboard of opponent pieces giving check
    pub fn find_checkers(&self) -> BB {
        let mut checkers = EMPTY_BB;
        // Find checking pawns
        checkers |= self.their_checking_pawns();
        // Find checkers along the files and ranks
        checkers |= self.file_and_rank_checkers();
        // Find checkers along the diagonals
        checkers |= self.diag_and_adiag_checkers();
        // Find knight checkers
        checkers |= self.knight_checkers();
        checkers
    }

    #[inline(always)]
    fn file_and_rank_checkers(&self) -> BB {
        let pseudo_attacks = self.our_pieces().king.lookup_rook_attacks(self.data.occ);
        pseudo_attacks & (self.their_pieces().rook | self.their_pieces().queen)
    }

    #[inline(always)]
    fn diag_and_adiag_checkers(&self) -> BB {
        let pseudo_attacks = self.our_pieces().king.lookup_bishop_attacks(self.data.occ);
        pseudo_attacks & (self.their_pieces().bishop | self.their_pieces().queen)
    }
    
    #[inline(always)]
    fn knight_checkers(&self) -> BB {
        let pseudo_attacks = self.our_pieces().king.lookup_knight_attacks();
        pseudo_attacks & self.their_pieces().knight
    }

    /// Return a bitboard of all pinned pieces
    pub fn pinned_pieces(&self) -> BB {

        let (king, their_pieces) = (self.our_pieces().king, self.their_pieces());
        let file_rank_pieces = their_pieces.rook | their_pieces.queen;
        let diag_adiag_pieces = their_pieces.bishop | their_pieces.queen;
        let occ = self.data.occ;

        // Pinned pieces are located where a king's "attack ray" meets an
        // attacking piece's attack ray, cast along the same axis
        let file_pins = king.hyp_quint(occ, Axis::File) & file_rank_pieces.file_attacks(occ);
        let rank_pins = king.hyp_quint(occ, Axis::Rank) & file_rank_pieces.rank_attacks(occ);
        let diag_pins = king.hyp_quint(occ, Axis::Diagonal) & diag_adiag_pieces.diag_attacks(occ);
        let adiag_pins = king.hyp_quint(occ, Axis::AntiDiagonal) & diag_adiag_pieces.adiag_attacks(occ);

        file_pins | rank_pins | diag_pins | adiag_pins
    }

    /// Identify which opponent piece is a particular position as the index
    /// of the array representation of the pieceset
    pub fn their_piece_at(&self, bb: BB) -> usize {
        debug_assert!(bb.pop_count() == 1);
        let their_piece_array = self.their_pieces().as_array();
        for piece in 1..7 {
            if (their_piece_array[piece] & bb).is_any() {
                return piece
            }
        }
        panic!(
            "their_piece_at could not locate the requested bit {}",
            bb.to_index()
        );
    }

    /// Identify which of our pieces is a particular position as the index
    /// of the array representation of the pieceset
    pub fn our_piece_at(&self, bb: BB) -> usize {
        debug_assert!(bb.pop_count() == 1);
        let our_piece_array = self.our_pieces().as_array();
        for piece in 1..7 {
            if (our_piece_array[piece] & bb).is_any() {
                return piece
            }
        }
        panic!(
            "their_piece_at could not locate the requested bit {}",
            bb.to_index()
        );
    }

    /// Identify if the piece at the specified square is a sliding piece
    pub fn their_piece_at_is_slider(&self, n: BB) -> bool {
        matches!(self.their_piece_at(n), 2 | 4 | 5) 
    }

    /// Convert to string representation for printing to the standard output
    pub fn to_string(&self) -> String {
        let mut array: [[char; 8]; 8] = [[' '; 8]; 8];
        let w_array = self.data.w_pieces.as_array();
        let b_array = self.data.b_pieces.as_array();
        for i in 1..7 {
            let mut char;
            match i {
                1 => char = 'p',
                2 => char = 'r',
                3 => char = 'n',
                4 => char = 'b',
                5 => char = 'q',
                6 => char = 'k',
                _ => char = ' ',
            };
            for bit in b_array[i].forward_scan() {
                let index = bit.to_index();
                let x = index / 8;
                let y = index % 8;
                array[x][y] = char;
            }
            char.make_ascii_uppercase();
            for bit in w_array[i].forward_scan() {
                let index = bit.to_index();
                let x = index / 8;
                let y = index % 8;
                array[x][y] = char;
            }
        };
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
            "|\n   --- --- --- --- --- --- --- --- \n    a   b   c   d   e   f   g   h "
        );
        return out
    }
}