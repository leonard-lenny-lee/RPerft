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
    pub fn promotion_rank(&self) -> u64 {
        self.state.promotion_rank()
    }
    /// Return the en passant capture rank mask
    pub fn ep_capture_rank(&self) -> u64 {
        self.state.ep_capture_rank()
    }
    /// Return the color of our pieces
    pub fn color(&self) -> Color {
        self.state.color()
    }
    pub fn our_ks_rook_starting_sq(&self) -> u64 {
        self.state.our_ks_rook_starting_sq()
    }
    pub fn our_qs_rook_starting_sq(&self) -> u64 {
        self.state.our_qs_rook_starting_sq()
    }
    pub fn their_ks_rook_starting_sq(&self) -> u64 {
        self.state.their_ks_rook_starting_sq()
    }
    pub fn their_qs_rook_starting_sq(&self) -> u64 {
        self.state.their_qs_rook_starting_sq()
    }
    /// Return the single push target squares of our pawns
    pub fn pawn_sgl_push_targets(&self) -> u64 {
        self.state.pawn_sgl_push_targets(self)
    }
    /// Return the double push target squares of our pawns
    pub fn pawn_dbl_push_targets(&self) -> u64 {
        self.state.pawn_dbl_push_targets(self)
    }
    /// Return the left capture target squares of our pawns
    pub fn pawn_lcap_targets(&self) -> u64 {
        self.state.pawn_lcap_targets(self)
    }
    /// Return the right capture target squares of our pawns
    pub fn pawn_rcap_targets(&self) -> u64 {
        self.state.pawn_rcap_targets(self)
    }
    /// Return the single push pawn sources from a map of target squares
    pub fn pawn_sgl_push_srcs(&self, targets: u64) -> u64 {
        self.state.pawn_sgl_push_srcs(targets)
    }
    /// Return the double push pawn sources from a map of target squares
    pub fn pawn_dbl_push_srcs(&self, targets: u64) -> u64 {
        self.state.pawn_dbl_push_srcs(targets)
    }
    /// Return the left capture pawn sources from a map of target squares
    pub fn pawn_lcap_srcs(&self, targets: u64) -> u64 {
        self.state.pawn_lcap_srcs(targets)
    }
    /// Return the right capture pawn sources from a map of target squares
    pub fn pawn_rcap_srcs(&self, targets: u64) -> u64 {
        self.state.pawn_rcap_srcs(targets)
    }
    /// Return the en passant source squares of our pieces
    pub fn pawn_en_passant_srcs(&self) -> u64 {
        self.state.pawn_en_passant_srcs(self)
    }
    /// Return the square of the piece being captured by en passant
    pub fn pawn_en_passant_cap(&self) -> u64 {
        self.state.pawn_en_passant_cap(self)
    }
    /// Return the mask of the squares the king must traverse to castle 
    /// kingside
    pub fn kingside_castle_mask(&self) -> u64 {
        self.state.kscm()
    }
    /// Return the mask of the squares the king must traverse to castle
    /// queenside so must be safe
    pub fn queenside_castle_mask_safe(&self) -> u64 {
        self.state.qscms()
    }
    /// Return the mask of the squares in between the king and the rook which
    /// must be free in order to castle
    pub fn queenside_castle_mask_free(&self) -> u64 {
        self.state.qscmf()
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
    pub fn unsafe_squares_pawn(&self) -> u64 {
        self.state.unsafe_squares_pawn(self)
    }
    /// Locate their pawns checking our king
    pub fn their_checking_pawns(&self) -> u64 {
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
    
}
