use super::*;
use movelist::Move;
use movelist::MoveType::{self, *};
use position::Position;

impl Position {
    /// Create a new position by applying move data to a position
    pub fn make_move(&self, mv: &Move) -> Position {
        // Create a copy of the current position to modify
        let mut new_pos = *self;

        // Unpack move data
        let target = mv.target();
        let src = mv.src();
        let moved_piece = new_pos.our_piece_at(src);
        let movetype = mv.movetype();

        // Common operations for all moves
        // Source squares must be free after a move and target squares must be occupied
        new_pos.free_squares |= src;
        new_pos.free_squares &= !target;

        // Our bitboards must be flipped at the target and source
        let our_pieces = new_pos.mut_our_pieces();
        let move_mask = src | target;
        our_pieces[moved_piece] ^= move_mask;
        our_pieces.all ^= move_mask;

        // If our king has moved, remove all further rights to castle
        if matches!(moved_piece, Piece::King) {
            new_pos.castling_rights &= !new_pos.our_backrank();
        }
        // If the rooks has moved from its starting square, remove rights to castle
        new_pos.castling_rights &= !src;

        // Increment half move clock; reset if capture or pawn move
        if mv.is_capture() || matches!(moved_piece, Piece::Pawn) {
            new_pos.halfmove_clock = 0
        } else {
            new_pos.halfmove_clock += 1
        }

        // Increment full move clock if black has moved
        new_pos.fullmove_clock += !new_pos.white_to_move() as u8;

        // Set en passant target sq to empty, this will be set to a value only
        // if the move was a pawn double push
        new_pos.en_passant_target_square = EMPTY_BB;

        // Execute special actions
        match movetype {
            Quiet => (),
            DoublePawnPush => new_pos.execute_double_push(target),
            ShortCastle | LongCastle => new_pos.execute_castle(movetype, target),
            Capture => new_pos.execute_capture(target),
            EnPassant => new_pos.execute_en_passant(target),
            KnightPromo => new_pos.execute_promotions(Piece::Knight, target),
            BishopPromo => new_pos.execute_promotions(Piece::Bishop, target),
            RookPromo => new_pos.execute_promotions(Piece::Rook, target),
            QueenPromo => new_pos.execute_promotions(Piece::Queen, target),
            KnightPromoCapture => {
                new_pos.execute_capture(target);
                new_pos.execute_promotions(Piece::Knight, target);
            }
            BishopPromoCapture => {
                new_pos.execute_capture(target);
                new_pos.execute_promotions(Piece::Bishop, target);
            }
            RookPromoCapture => {
                new_pos.execute_capture(target);
                new_pos.execute_promotions(Piece::Rook, target);
            }
            QueenPromoCapture => {
                new_pos.execute_capture(target);
                new_pos.execute_promotions(Piece::Queen, target);
            }
        }
        new_pos.occupied_squares = !new_pos.free_squares;
        // Change the turn and state
        new_pos.change_state();
        new_pos.update_key(moved_piece.value(), src, target, self);
        return new_pos;
    }

    #[inline(always)]
    fn execute_capture(&mut self, target: BB) {
        let captured_piece = self.their_piece_at(target);
        let their_pieces = self.mut_their_pieces();
        // If capture has taken place, then their bitboard must be unset at
        // target positions
        their_pieces[captured_piece] ^= target;
        their_pieces.all ^= target;
        // If their rook has been captured, check if it's a rook from on their
        // starting square. If so, unset their corresponding castling right
        self.castling_rights &= !target;
        // Update Zobrist hash with the capture
        self.update_square(captured_piece as usize, target, !self.white_to_move())
    }

    #[inline(always)]
    fn execute_promotions(&mut self, promotion_piece: Piece, target: BB) {
        let our_pieces = self.mut_our_pieces();
        our_pieces[promotion_piece] ^= target;
        // Unset the pawn from our pawn bitboard
        our_pieces[Piece::Pawn] ^= target;
        // Update the Zobrist hashes
        self.update_square(Piece::Pawn.value(), target, self.white_to_move());
        self.update_square(promotion_piece.value(), target, self.white_to_move());
    }

    #[inline(always)]
    fn execute_castle(&mut self, movetype: MoveType, target: BB) {
        let our_pieces = self.mut_our_pieces();
        // For castling moves, we also need the update our rook and shared
        // bitboards
        let (rook_src, rook_target) = match movetype {
            // For short castle, the rook has transported from a
            // position one east of the target square to one west
            ShortCastle => (target.east_one(), target.west_one()),
            // For the long castle, the rook has transported from two squares
            // west of the target to one square east
            LongCastle => (target.west_two(), target.east_one()),
            _ => return,
        };
        let castle_mask = rook_src | rook_target;
        our_pieces[Piece::Rook] ^= castle_mask;
        our_pieces[Piece::All] ^= castle_mask;
        // We also need to updated shared bitboards
        self.occupied_squares ^= castle_mask;
        self.free_squares ^= castle_mask;
        // Update the Zobrist hash for the rook movement
        self.update_moved_piece(
            Piece::Rook.value(),
            rook_src,
            rook_target,
            self.white_to_move(),
        )
    }

    #[inline(always)]
    fn execute_en_passant(&mut self, target: BB) {
        // If white made the en passant capture, then the square at which the
        // capture takes place is on square south of the target square and the
        // opposite for black
        let ep_capture_sq = self.pawn_sgl_push_srcs(target);
        // Reflect the capture on the opponent bitboards
        let their_pieces = self.mut_their_pieces();
        their_pieces[Piece::Pawn] ^= ep_capture_sq;
        their_pieces[Piece::All] ^= ep_capture_sq;
        // We also need to update shared bitboards
        self.occupied_squares ^= ep_capture_sq;
        self.free_squares ^= ep_capture_sq;
        // Update Zobrist hash
        self.update_square(Piece::Pawn.value(), ep_capture_sq, !self.white_to_move())
    }

    #[inline(always)]
    fn execute_double_push(&mut self, target: BB) {
        // If white made the double pawn push, then the ep target
        // square must be one square south of the target square and vice versa
        // for black
        let en_passant_target = self.pawn_sgl_push_srcs(target);
        self.en_passant_target_square = en_passant_target
    }
}
