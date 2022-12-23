use super::*;

/// Turn specific logic, methods and data are delegated to the White and Black
/// structs which implement the State trait

pub struct White;
pub struct Black;

impl Position {

    pub fn change_state(&mut self) {
        self.data.white_to_move = !self.data.white_to_move;
        self.state = self.state.change_state();
    }

    /// Return the piece set of our pieces
    pub fn our_pieces(&self) -> &PieceSet {
        self.state.our_pieces(&self.data)
    }
    /// Return the piece set of their pieces
    pub fn their_pieces(&self) -> &PieceSet {
        self.state.their_pieces(&self.data)
    }
    /// Return the promotion rank mask
    pub fn promotion_rank(&self) -> BB {
        self.state.promotion_rank()
    }
    /// Return our backrank
    pub fn our_backrank(&self) -> BB {
        self.state.our_backrank()
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
        self.state.pawn_sgl_push_targets(&self.data)
    }
    /// Return the double push target squares of our pawns
    pub fn pawn_dbl_push_targets(&self) -> BB {
        self.state.pawn_dbl_push_targets(&self.data)
    }
    /// Return the left capture target squares of our pawns
    pub fn pawn_lcap_targets(&self) -> BB {
        self.state.pawn_lcap_targets(&self.data)
    }
    /// Return the right capture target squares of our pawns
    pub fn pawn_rcap_targets(&self) -> BB {
        self.state.pawn_rcap_targets(&self.data)
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
        self.state.pawn_en_passant_srcs(&self.data)
    }
    /// Return the square of the piece being captured by en passant
    pub fn pawn_en_passant_cap(&self) -> BB {
        self.state.pawn_en_passant_capture_square(&self.data)
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
        self.state.our_kingside_castle(&self.data)
    }
    /// Return the queenside castling rights
    pub fn our_queenside_castle(&self) -> bool {
        self.state.our_queenside_castle(&self.data)
    }
    /// Return all the squares attacked by their pawns
    pub fn unsafe_squares_pawn(&self) -> BB {
        self.state.unsafe_squares_pawn(&self.data)
    }
    /// Locate their pawns checking our king
    pub fn their_checking_pawns(&self) -> BB {
        self.state.pawn_checking_squares(&self.data) 
        & self.their_pieces().pawn
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

pub(crate) trait State {
    /// Current state pointer
    fn current_state(&self) -> Box<dyn State>;
    /// Other state pointer
    fn change_state(&self) -> Box<dyn State>;
    /// Pawns starting on this rank promote
    fn promotion_rank(&self) -> BB;
    /// Pawns on this rank can capture en passant
    fn ep_capture_rank(&self) -> BB;
    /// Our back rank
    fn our_backrank(&self) -> BB;
    /// Our pieceset
    fn our_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet;
    /// Their pieceset
    fn their_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet;
    /// Our color
    fn color(&self) -> Color;
    /// Single push towards the opponent end
    fn pawn_sgl_push(&self, src: BB) -> BB;
    /// Double push towards the opponent end
    fn pawn_dbl_push(&self, src: BB) -> BB;
    /// Left capture towards the opponent end
    fn pawn_left_capture(&self, src: BB) -> BB;
    /// Right capture towards the opponent end
    fn pawn_right_capture(&self, src: BB) -> BB;
    /// Find our pawns' push target squares
    fn pawn_sgl_push_targets(&self, data: &Data) -> BB;
    /// Find our pawns' double push target squares
    fn pawn_dbl_push_targets(&self, data: &Data) -> BB;
    /// Find our pawns' left capture target squares
    fn pawn_lcap_targets(&self, data: &Data) -> BB;
    /// Find our pawns' right capture target squares
    fn pawn_rcap_targets(&self, data: &Data) -> BB;
    /// Find our pawns capable of pushing to the single push target squares
    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB;
    /// Find our pawns capable of pushing to the double push target squares
    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB;
    /// Find our pawns capable of moving to the left capture target squares
    fn pawn_lcap_srcs(&self, targets: BB) -> BB;
    /// Find our pawns capable of moving to the right capture target squares
    fn pawn_rcap_srcs(&self, targets: BB) -> BB;
    /// Find our pawns capable of moving to the en passant capture square
    fn pawn_en_passant_srcs(&self, data: &Data) -> BB;
    /// Find the square on which en passant capture occurs
    fn pawn_en_passant_capture_square(&self, data: &Data) -> BB;
    /// The squares that must be free and safe to short castle
    fn kingside_castle_mask(&self) -> BB;
    /// The squares that must be safe to long castle
    fn queenside_castle_mask_safe(&self) -> BB;
    /// The squares that must be free to long castle
    fn queenside_castle_mask_free(&self) -> BB;
    /// Our kingside castle right
    fn our_kingside_castle(&self, data: &Data) -> bool;
    /// Our queenside castle right
    fn our_queenside_castle(&self, data: &Data) -> bool;
    /// Squares rendered unsafe by their pawn attacks
    fn unsafe_squares_pawn(&self, data: &Data) -> BB;
    /// Squares which if occupied by a pawn, is putting us in check
    fn pawn_checking_squares(&self, data: &Data) -> BB;
    /// Square our kingside rook starts on
    fn our_ks_rook_starting_sq(&self) -> BB;
    /// Square our queenside rook starts on
    fn our_qs_rook_starting_sq(&self) -> BB;
    /// Square their kingside rook starts on
    fn their_ks_rook_starting_sq(&self) -> BB;
    /// Square their queenside rook starts on
    fn their_qs_rook_starting_sq(&self) -> BB;
    /// Set our kingside castle right
    /// Return a mutable reference to our pieceset
    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;
    /// Return a mutable reference to their pieceset
    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet;

}

impl State for White {

    fn current_state(&self) -> Box<dyn State> {
        Box::new(White)
    }

    fn change_state(&self) -> Box<dyn State> {
        Box::new(Black)
    }

    fn promotion_rank(&self) -> BB { 
        RANK_7 
    }

    fn ep_capture_rank(&self) -> BB {
        RANK_5
    }

    fn our_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet {
        &data.w_pieces
    }

    fn their_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet {
        &data.b_pieces
    }

    fn our_kingside_castle(&self, data: &Data) -> bool {
        (data.castling_rights & W_KINGSIDE_ROOK_STARTING_SQ).is_any()
    }

    fn our_queenside_castle(&self, data: &Data) -> bool {
        (data.castling_rights & W_QUEENSIDE_ROOK_STARTING_SQ).is_any()
    }

    fn our_backrank(&self) -> BB {
        RANK_1
    }

    fn kingside_castle_mask(&self) -> BB {
        BB(0x60)
    }

    fn queenside_castle_mask_safe(&self) -> BB {
        BB(0xc)
    }

    fn queenside_castle_mask_free(&self) -> BB {
        BB(0xe)
    }

    fn color(&self) -> Color {
        Color::White
    }

    fn our_ks_rook_starting_sq(&self) -> BB {
        W_KINGSIDE_ROOK_STARTING_SQ
    }

    fn our_qs_rook_starting_sq(&self) -> BB {
        W_QUEENSIDE_ROOK_STARTING_SQ
    }

    fn their_ks_rook_starting_sq(&self) -> BB {
        B_KINGSIDE_ROOK_STARTING_SQ
    }

    fn their_qs_rook_starting_sq(&self) -> BB {
        B_QUEENSIDE_ROOK_STARTING_SQ
    }


    fn pawn_sgl_push(&self, src: BB) -> BB {
        src.north_one()
    }

    fn pawn_dbl_push(&self, src: BB) -> BB {
        src.north_two()
    }

    fn pawn_left_capture(&self, src: BB) -> BB {
        src.nort_west()
    }

    fn pawn_right_capture(&self, src: BB) -> BB {
        src.nort_east()
    }

    fn pawn_sgl_push_targets(&self, data: &Data) -> BB {
        data.w_pieces.pawn.north_one() & data.free
    }

    fn pawn_dbl_push_targets(&self, data: &Data) -> BB {
        let sgl_push = (data.w_pieces.pawn & RANK_2).north_one() & data.free;
        sgl_push.north_one() & data.free
    }

    fn pawn_lcap_targets(&self, data: &Data) -> BB {
        data.w_pieces.pawn.nort_west() & data.b_pieces.any
    }

    fn pawn_rcap_targets(&self, data: &Data) -> BB {
        data.w_pieces.pawn.nort_east() & data.b_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        targets.south_one()
    }

    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        targets.south_two()
    }

    fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        targets.sout_east()
    }

    fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        targets.sout_west()
    }

    fn pawn_en_passant_srcs(&self, data: &Data) -> BB {
        (data.en_passant_target_sq.sout_west()
            | data.en_passant_target_sq.sout_east())
            & data.w_pieces.pawn
            & RANK_5
    }

    fn pawn_en_passant_capture_square(&self, data: &Data) -> BB {
        data.en_passant_target_sq.south_one()
    }

    fn unsafe_squares_pawn(&self, data: &Data) -> BB {
        data.b_pieces.pawn.sout_west() | data.b_pieces.pawn.sout_east()
    }

    fn pawn_checking_squares(&self, data: &Data) -> BB {
        let king = data.w_pieces.king;
        king.nort_east() | king.nort_west()
    }

    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet {
        &mut data.w_pieces
    }

    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet{
        &mut data.b_pieces
    }

}

impl State for Black {

    fn current_state(&self) -> Box<dyn State> {
        Box::new(Black)
    }

    fn change_state(&self) -> Box<dyn State> {
        Box::new(White)
    }

    fn promotion_rank(&self) -> BB {
        RANK_2
    }

    fn ep_capture_rank(&self) -> BB {
        RANK_4
    }

    fn our_backrank(&self) -> BB {
        RANK_8
    }

    fn our_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet {
        &data.b_pieces
    }

    fn their_pieces<'a>(&'a self, data: &'a Data) -> &PieceSet {
        &data.w_pieces
    }

    fn our_kingside_castle(&self, data: &Data) -> bool {
        (data.castling_rights & B_KINGSIDE_ROOK_STARTING_SQ).is_any()
    }

    fn our_queenside_castle(&self, data: &Data) -> bool {
        (data.castling_rights & B_QUEENSIDE_ROOK_STARTING_SQ).is_any()
    }

    fn kingside_castle_mask(&self) -> BB {
        BB(0x6000000000000000)
    }

    fn queenside_castle_mask_safe(&self) -> BB {
        BB(0xc00000000000000)
    }

    fn queenside_castle_mask_free(&self) -> BB {
        BB(0xe00000000000000)
    }

    fn color(&self) -> Color {
        Color::Black
    }

    fn our_ks_rook_starting_sq(&self) -> BB {
        B_KINGSIDE_ROOK_STARTING_SQ
    }

    fn our_qs_rook_starting_sq(&self) -> BB {
        B_QUEENSIDE_ROOK_STARTING_SQ
    }

    fn their_ks_rook_starting_sq(&self) -> BB {
        W_KINGSIDE_ROOK_STARTING_SQ
    }

    fn their_qs_rook_starting_sq(&self) -> BB {
        W_QUEENSIDE_ROOK_STARTING_SQ
    }


    fn pawn_sgl_push(&self, src: BB) -> BB {
        src.south_one()
    }

    fn pawn_dbl_push(&self, src: BB) -> BB {
        src.south_two()
    }

    fn pawn_left_capture(&self, src: BB) -> BB {
        src.sout_west()
    }

    fn pawn_right_capture(&self, src: BB) -> BB {
        src.sout_east()
    }

    fn pawn_sgl_push_targets(&self, data: &Data) -> BB {
        data.b_pieces.pawn.south_one() & data.free
    }

    fn pawn_dbl_push_targets(&self, data: &Data) -> BB {
        let sgl_push = (data.b_pieces.pawn & RANK_7).south_one() & data.free;
        sgl_push.south_one() & data.free
    }

    fn pawn_lcap_targets(&self, data: &Data) -> BB {
        data.b_pieces.pawn.sout_west() & data.w_pieces.any
    }

    fn pawn_rcap_targets(&self, data: &Data) -> BB {
        data.b_pieces.pawn.sout_east() & data.w_pieces.any
    }

    fn pawn_sgl_push_srcs(&self, targets: BB) -> BB {
        targets.north_one()
    }

    fn pawn_dbl_push_srcs(&self, targets: BB) -> BB {
        targets.north_two()
    }

    fn pawn_lcap_srcs(&self, targets: BB) -> BB {
        targets.nort_east()
    }

    fn pawn_rcap_srcs(&self, targets: BB) -> BB {
        targets.nort_west()
    }

    fn pawn_en_passant_srcs(&self, data: &Data) -> BB {
        (data.en_passant_target_sq.nort_west()
        | data.en_passant_target_sq.nort_east())
        & data.b_pieces.pawn
        & RANK_4
    }

    fn pawn_en_passant_capture_square(&self, data: &Data) -> BB {
        data.en_passant_target_sq.north_one()
    }

    fn unsafe_squares_pawn(&self, data: &Data) -> BB {
        data.w_pieces.pawn.nort_west() | data.w_pieces.pawn.nort_east()
    }

    fn pawn_checking_squares(&self, data: &Data) -> BB {
        let king = data.b_pieces.king;
        king.sout_east() | king.sout_west()
    }

    fn mut_our_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet {
        &mut data.b_pieces
    }

    fn mut_their_pieces<'a>(&'a self, data: &'a mut Data) -> &'a mut PieceSet{
        &mut data.w_pieces
    }

}