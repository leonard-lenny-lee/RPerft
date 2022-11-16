
mod search;

fn draw_bitboard(n: u64) {
    let mut out = String::new();
    for i in 0..64 {
        if i % 8 == 0 {
            out.push_str("\n")
        }
        if ((1 << (7 - i / 8) * 8 + (i % 8)) & n) != 0 {
            out.push('1')
        } else {
            out.push('0')
        }
    }
    println!("{}", out);
}

fn main() {
    let tables = search::LookupTables{
        rank_masks: search::generate_rank_masks(),
        file_masks: search::generate_file_masks(),
        diagonal_masks: search::generate_diagonal_masks(),
        antidiagonal_masks: search::generate_antidiagonal_masks(),
    };
    let h_masks = search::generate_rank_masks();
    let v_masks = search::generate_file_masks();
    let d_masks = search::generate_diagonal_masks();
    let a_masks = search::generate_antidiagonal_masks();
    let occ: u64 = 0b00010100000000001100010100000000;
    let s: u64 =   0b0000010000000000;
    let tables_ref = &tables;
    let rank_attacks = search::hyp_quint(occ, s, &tables_ref.rank_masks);
    let file_attacks = search::hyp_quint(occ, s, &v_masks);
    let diagonal_attacks = search::hyp_quint(occ, s, &d_masks);
    let adiagonal_attacks = search::hyp_quint(occ, s, &a_masks);
    let attacks = rank_attacks | file_attacks | diagonal_attacks | adiagonal_attacks;
    draw_bitboard(attacks)
}