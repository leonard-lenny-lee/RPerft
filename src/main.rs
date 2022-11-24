// use chess_engine::*;
// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    
    // let fen = "";
    // let state = init(fen);
    let mut x: u64 = 64;
    test(&mut x);

    println!("{}", x);
}

fn test(x: &mut u64) {
    let y: u64 = 128;
    *x |= y
}