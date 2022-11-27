use chess_engine::*;
// fn init(fen: Option<String>) -> global::State {
//     let ctx = global::State::new_from_fen(common::DEFAULT_FEN.to_string());
//     return ctx;
// }
fn main() {
    let attacker: u64 = 1 << 48;
    let king = 1 << 20;
    let string = common::bittools::bitboard_to_string(
        common::bittools::connect_squares(attacker, king)
    );
    print!("{}", string)
}
