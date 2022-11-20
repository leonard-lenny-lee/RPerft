use chess_engine::*;
// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    
    // let fen = "";
    // let state = init(fen);
    let pos = position::Position::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let maps = global::maps::Maps::new();
    let result = search::depth_search::perft(pos, 1, &maps);
    println!("{}", result);
}