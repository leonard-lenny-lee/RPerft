

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

// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    
    // let fen = "";
    // let state = init(fen);
    let x: u64 = 1 << 27;
    let y: u64 = 1 << 9;
    draw_bitboard(chess_engine::common::bittools::ray_axis(x, y));
}