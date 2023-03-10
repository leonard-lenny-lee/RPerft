use chess::*;
use uci::*;

fn main() {
    env_logger::init();
    let mut state = state::State::init();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match CommandNode::parse(input) {
            Ok(c) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!("Commands parsed");
                    c.print_parse_tree(1);
                }
                if c.quit() {
                    return;
                }
                if let Err(e) = c.execute(&mut state) {
                    e.warn()
                };
            }
            Err(e) => e.warn(),
        };
    }
}
