use test_case::test_case;

use super::*;
use position::Data;

#[test]
fn test_bitmask_to_algebraic() {
    let expected = "f8";
    let input = 1 << 61;
    let result = &bitmask_to_algebraic(input)[..];
    assert_eq!(expected, result)
}

#[test]
fn test_squares_to_bitboard() {
    let squares = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let bitboard = squares_to_bitboard(squares);
    assert_eq!(bitboard, RANK_1);
}

#[test]
fn test_get_lsb() {
    let bitboard = squares_to_bitboard(vec![19, 30]);
    let lsb = get_lsb(bitboard);
    assert_eq!(1 << 19, lsb);
}

#[test]
fn test_get_ilsb() {
    let bitboard = squares_to_bitboard(vec![41, 55]);
    let ilsb = ilsb(bitboard);
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

#[test_case(MAPS.rank, vec![17, 19, 23, 35], 19, vec![17, 18, 20, 21, 22, 23];"RANK")]
#[test_case(MAPS.file, vec![20, 44, 18], 20, vec![4, 12, 28, 36, 44];"FILE")]
#[test_case(MAPS.diag, vec![27, 54, 18], 27, vec![18, 36, 45, 54];"DIAG")]
#[test_case(MAPS.adiag, vec![6, 13, 34, 41, 43], 34, vec![41, 27, 20, 13];"ADIAG")]
fn test_hyp_quint(maps: [u64; 64], occ: Vec<i32>, slider: i32, expected: Vec<i32>) {
    let occ = squares_to_bitboard(occ);
    let slider = 1 << slider;
    let result = hyp_quint(occ, slider, &maps);
    let expected = squares_to_bitboard(expected);
    assert_eq!(result, expected);
}

#[test_case(north_one, 50, 58;"north")]
#[test_case(nort_east, 34, 43;"noea")]
fn test_shifts(func: fn(u64) -> u64, input: i32, expected: i32) {
    assert_eq!(func(1 << input), 1 << expected);
}

#[test_case(north_one, 59; "north")]
fn test_overflows(func: fn(u64) -> u64, input: i32) {
    assert_eq!(func(1 << input), 0);
}

#[ignore]
#[test]
fn test_position_to_string() {
    let data = Data::from_fen(DEFAULT_FEN.to_string());
    let out = piecesets_to_string(data.w_pieces, data.b_pieces);
    print!("{}", out)
}

#[ignore]
#[test]
fn test_kiwipete_to_string() {
    let kiwipete = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let data = Data::from_fen(kiwipete.to_string());
    let out = piecesets_to_string(data.w_pieces, data.b_pieces);
    print!("{}", out)
}

#[test]
fn test_flip_vertical() {
    let bb = 0x8040201;
    let result = flip_vertical(bb);
    assert_eq!(result, 0x102040800000000)
}