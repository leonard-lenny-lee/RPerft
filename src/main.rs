use chess_engine::*;

fn main() {
    let mut state = state::State::initalize();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match interface::Command::parse(input) {
            Ok(c) => {c.execute(&mut state);},
            Err(e) => e.warn(),
        };
    }
}
