
mod search_engine;
mod context;
mod mechanics;
mod evaluator;

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
    
fn init(fen: Option<String>) -> context::GameContext {
    let ctx = context::GameContext::new_from_fen(fen);
    return ctx;
}
fn main() {
    
    let fen = "";
    let ctx = init(fen);
    let x:u64 = 1<<60;
    draw_bitboard(x<<6);
}