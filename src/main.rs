use chess_engine::*;
use interface::*;

fn main() {
    let mut state = state::State::initalize();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match interface::Command::parse(input) {
            Ok(c) => {
                if let Err(e) = c.execute(&mut state) {
                    e.warn()
                };
            },
            Err(e) => e.warn(),
        };
    }
}
