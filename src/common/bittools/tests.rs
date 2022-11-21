mod tests {

    use crate::common::bittools::*;
    use crate::common::*;
    use crate::global::maps::Maps;
    use test_case::test_case;

    #[test]
    fn test_squares_to_bitboard() {
        let squares = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let bitboard = squares_to_bitboard(squares);
        assert_eq!(bitboard, RANK_1);
    }

    #[test]
    fn test_get_lsb() {
        let bitboard = squares_to_bitboard(vec![19, 30]);
        let lsb = get_lsb(&bitboard);
        assert_eq!(1 << 19, lsb);
    }

    #[test]
    fn test_get_ilsb() {
        let bitboard = squares_to_bitboard(vec![41, 55]);
        let ilsb = ilsb(&bitboard);
        assert_eq!(ilsb, 41);
    }

    #[test]
    fn test_forward_scan() {
        let scan_result = forward_scan(FILE_E);
        let expected: Vec<u64> = vec![
            1<<4, 1<<12, 1<<20, 1<<28, 1<<36, 1<<44, 1<<52, 1<<60
        ];
        assert_eq!(scan_result, expected);
    }

    #[test]
    fn test_hyp_quint_ranks() {
        let maps = Maps::new().rank;
        let occ = squares_to_bitboard(vec![17, 19, 23, 35]);
        let slider = 1 << 19;
        let attack_sq = hyp_quint(occ, slider, &maps);
        let expected = squares_to_bitboard(vec![17, 18, 20, 21, 22, 23]);
        assert_eq!(attack_sq, expected);
    }

    #[test]
    fn test_hyp_quint_files() {
        let maps = Maps::new().file;
        let occ = squares_to_bitboard(vec![20, 44, 18]);
        let slider = 1 << 20;
        let attack_sq = hyp_quint(occ, slider, &maps);
        let expected = squares_to_bitboard(vec![4, 12, 28, 36, 44]);
        assert_eq!(attack_sq, expected);
    }

    #[test]
    fn test_hyp_quint_diag() {
        let maps = Maps::new().diag;
        let occ = squares_to_bitboard(vec![27, 54, 18]);
        let slider = 1 << 27;
        let attack_sq = hyp_quint(occ, slider, &maps);
        let expected = squares_to_bitboard(vec![18, 36, 45, 54]);
        assert_eq!(attack_sq, expected);
    }

}