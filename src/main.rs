use chess_engine::*;
// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    
    // let fen = "";
    // let state = init(fen);
    let x = d!(chess_engine::common::Piece::Rook);
    println!("{}", x);
}