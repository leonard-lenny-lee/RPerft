use chess_engine::*;
// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    
    // let fen = "";
    // let state = init(fen);
    let x: u64 = 1 << 27;
    let y: u64 = 1 << 9;
    common::bittools::draw_bitboard(common::bittools::ray_axis(x, y));
}