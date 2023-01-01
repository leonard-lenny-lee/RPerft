use chess_engine::*;
use interface::*;

fn main() {
    let mut state = state::State::initalize();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match Command::parse(input) {
            Ok(c) => {
                if matches!(c.cmd, CommandType::Root(Root::Quit)) {
                    return
                }
                if let Err(e) = c.execute(&mut state) {
                    e.warn()
                };
            },
            Err(e) => e.warn(),
        };
    }
}
