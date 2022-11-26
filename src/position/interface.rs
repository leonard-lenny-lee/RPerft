/// Public interface methods for the Position Struct which accesses the state
/// methods

use super::*;

impl Position {

    pub fn new_from_fen(fen: String) -> Position {
        let data = Data::from_fen(fen);
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
    /// queenside
    pub fn queenside_castle_mask(&self) -> u64 {
        self.state.qscm()
    }
    /// Return the king side castling rights
    pub fn kingside_castle(&self) -> bool {
        self.state.ksc(self)
    }
    /// Return the queenside castling rights
    pub fn queenside_castle(&self) -> bool {
        self.state.qsc(self)
    }
}
