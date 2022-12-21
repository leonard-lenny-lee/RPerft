/// Public interface methods for the Position Struct which accesses the state
/// methods. This is a loose implementation of State pattern.

use super::*;

impl Position {

    pub fn new_from_fen(fen: String) -> Position {
        let data = Data::from_fen(fen);
        Position::new(data)
    }

    pub fn new(data: Data) -> Position {
        Position {
            data: data,
            state: (
                if data.white_to_move {
                    Box::new(White{})
                } else {
                    Box::new(Black{})
                }
            ) 
        }
    }

    pub fn change_state(&mut self) {
        self.data.white_to_move = !self.data.white_to_move;
        self.state = if self.data.white_to_move {
            Box::new(White{})
        } else {
            Box::new(Black{})
        } 
    }

    /// Return the piece set of our pieces
    pub fn our_pieces(&self) -> &PieceSet {
        self.state.our_pieces(self)
    }
    /// Return the piece set of their pieces
    pub fn their_pieces(&self) -> &PieceSet {
        self.state.their_pieces(self)
    }
    /// Return the promotion rank mask
    pub fn promotion_rank(&self) -> BB {
        self.state.promotion_rank()
    }
    /// Return the en passant capture rank mask
    pub fn ep_capture_rank(&self) -> BB {
        self.state.ep_capture_rank()
    }
    /// Return the color of our pieces
    pub fn color(&self) -> Color {
        self.state.color()
    }
    pub fn our_ks_rook_starting_sq(&self) -> BB {
        self.state.our_ks_rook_starting_sq()
    }
    pub fn our_qs_rook_starting_sq(&self) -> BB {
        self.state.our_qs_rook_starting_sq()
    }
    pub fn their_ks_rook_starting_sq(&self) -> BB {
        self.state.their_ks_rook_starting_sq()
    }
    pub fn their_qs_rook_starting_sq(&self) -> BB {
        self.state.their_qs_rook_starting_sq()
    }
    pub fn pawn_sgl_push(&self, src: BB) -> BB {
        self.state.pawn_sgl_push(src)
    }
    pub fn pawn_dbl_push(&self, src: BB) -> BB {
        self.state.pawn_dbl_push(src)
    }
    pub fn pawn_left_capture(&self, src: BB) -> BB {
        self.state.pawn_left_capture(src)
    }
    pub fn pawn_right_capture(&self, src: BB) -> BB {
        self.state.pawn_right_capture(src)
    }
    /// Return the single push target squares of our pawns
    pub fn pawn_sgl_push_targets(&self) -> BB {
        self.state.pawn_sgl_push_targets(self)
    }
    /// Return the double push target squares of our pawns
    pub fn pawn_dbl_push_targets(&self) -> BB {
        self.state.pawn_dbl_push_targets(self)
    }
    /// Return the left capture target squares of our pawns
    pub fn pawn_lcap_targets(&self) -> BB {
        self.state.pawn_lcap_targets(self)
    }
    /// Return the right capture target squares of our pawns
    pub fn pawn_rcap_targets(&self) -> BB {
        self.state.pawn_rcap_targets(self)
    }
    /// Return the single push pawn sources from a map of target squares
    pub fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        self.state.pawn_sgl_push_srcs(targets)
    }
    /// Return the double push pawn sources from a map of target squares
    pub fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        self.state.pawn_dbl_push_srcs(targets)
    }
    /// Return the left capture pawn sources from a map of target squares
    pub fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        self.state.pawn_lcap_srcs(targets)
    }
    /// Return the right capture pawn sources from a map of target squares
    pub fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        self.state.pawn_rcap_srcs(targets)
    }
    /// Return the en passant source squares of our pieces
    pub fn pawn_en_passant_srcs(&self) -> BB {
        self.state.pawn_en_passant_srcs(self)
    }
    /// Return the square of the piece being captured by en passant
    pub fn pawn_en_passant_cap(&self) -> BB {
        self.state.pawn_en_passant_cap(self)
    }
    /// Return the mask of the squares the king must traverse to castle 
    /// kingside
    pub fn kingside_castle_mask(&self) -> BB {
        self.state.kingside_castle_mask()
    }
    /// Return the mask of the squares the king must traverse to castle
    /// queenside so must be safe
    pub fn queenside_castle_mask_safe(&self) -> BB {
        self.state.queenside_castle_mask_safe()
    }
    /// Return the mask of the squares in between the king and the rook which
    /// must be free in order to castle
    pub fn queenside_castle_mask_free(&self) -> BB {
        self.state.queenside_castle_mask_free()
    }
    /// Return our king side castling rights
    pub fn our_kingside_castle(&self) -> bool {
        self.state.our_ksc(self)
    }
    /// Return the queenside castling rights
    pub fn our_queenside_castle(&self) -> bool {
        self.state.our_sqc(self)
    }
    /// Return their king side castling rights
    pub fn their_kingside_castle(&self) -> bool {
        self.state.their_ksc(self)
    }
    /// Return their queenside castling rights
    pub fn their_queenside_castle(&self) -> bool {
        self.state.their_qsc(self)
    }
    /// Return all the squares attacked by their pawns
    pub fn unsafe_squares_pawn(&self) -> BB {
        self.state.unsafe_squares_pawn(self)
    }
    /// Locate their pawns checking our king
    pub fn their_checking_pawns(&self) -> BB {
        self.state.pawn_checking_squares(self) 
        & self.their_pieces().pawn
    }
    /// Set our kingside castle permission
    pub fn set_our_ksc(&mut self, value: bool) {
        self.state.set_our_ksc(&mut self.data, value)
    }
    /// Set our queenside castle permission
    pub fn set_our_qsc(&mut self, value: bool) {
        self.state.set_our_qsc(&mut self.data, value)
    }
    /// Set their kingside castle permission
    pub fn set_their_ksc(&mut self, value: bool) {
        self.state.set_their_ksc(&mut self.data, value)
    }
    /// Set their queenside castle permission
    pub fn set_their_qsc(&mut self, value: bool) {
        self.state.set_their_qsc(&mut self.data, value)
    }
    /// Return our piece set as a mutable reference
    pub fn mut_our_pieces(&mut self) -> &mut PieceSet {
        self.state.mut_our_pieces(&mut self.data)
    }
    /// Return their piece set as a mutable reference
    pub fn mut_their_pieces(&mut self) -> &mut PieceSet {
        self.state.mut_their_pieces(&mut self.data)
    }

    /// Identify which opponent piece is a particular position as the index
    /// of the array representation of the pieceset
    pub fn their_piece_at(&self, bb: BB) -> usize {
        debug_assert!(bb.pop_count() == 1);
        let their_piece_array = self.their_pieces().as_array();
        for piece in 1..7 {
            if their_piece_array[piece] & bb != EMPTY_BB {
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
            if our_piece_array[piece] & bb != EMPTY_BB {
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
