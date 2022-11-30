use super::*;

impl Position {

    /// Generate a Zobrist hash key to encode the position. Used for 
    /// the transposition table
    pub fn zobrist_hash(&self, array: &[u64; 781]) -> u64 {
        self.hash_board(array) ^ self.hash_castling(array)
        ^ self.hash_en_passant(array) ^ self.hash_turn(array)
    }

    fn hash_board(&self, array: &[u64; 781]) -> u64 {
        let mut hash = 0;
        let pieces = [
            self.data.b_pieces.as_hash_array(),
            self.data.w_pieces.as_hash_array()
        ];
        for (color_idx, color_arr) in pieces.iter().enumerate() {
            for (piece_idx, piece_bb) in color_arr.iter().enumerate() {
                for bit in bt::forward_scan(*piece_bb) {
                    let bit_index = bt::ilsb(bit);
                    let piece_id = piece_idx * 2 + color_idx;
                    let hash_array_index = 64 * piece_id + bit_index;
                    hash ^= array[hash_array_index]
                }
            }
        }
        return hash;
    }

    fn hash_castling(&self, array: &[u64; 781]) -> u64 {
        let mut hash = 0;
        if self.data.w_kingside_castle {
            hash ^= array[768]
        }
        if self.data.w_queenside_castle {
            hash ^= array[769]
        }
        if self.data.b_kingside_castle {
            hash ^= array[770]
        }
        if self.data.b_queenside_castle {
            hash ^= array[771]
        }
        return hash;
    }

    fn hash_en_passant(&self, array: &[u64; 781]) -> u64 {
        if self.state.pawn_en_passant_srcs(self) != EMPTY_BB {
            return array[772 + bt::ilsb(self.data.en_passant_target_sq) % 8]
        }
        return 0
    }

    fn hash_turn(&self, array: &[u64; 781]) -> u64 {
        if self.data.white_to_move {
            return array[780]
        }
        return 0
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 0x463b96181691fc9c; "1")]
    #[test_case("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", 0x823c9b50fd114196; "2")]
    #[test_case("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", 0x0756b94461c50fb0; "3")]
    #[test_case("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2", 0x662fafb965db29d4; "4")]
    #[test_case("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", 0x22a48b5a8e47ff78; "5")]
    #[test_case("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3", 0x652a607ca3f242c1; "6")]
    #[test_case("rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4", 0x00fdd303c946bdd9; "7")]
    #[test_case("rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3", 0x3c8123ea7b067637; "8")]
    #[test_case("rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4", 0x5c3f9b829b279560; "9")]
    fn test_zobrist_polyglot_hash(fen: &str, expected_hash: u64) {
        let pos = Position::new_from_fen(fen.to_string());
        let hash = pos.zobrist_hash(&POLYGLOT_RANDOM_ARRAY);
        assert_eq!(hash, expected_hash);
    }
}
