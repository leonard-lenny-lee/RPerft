use chess::*;
use engine::Engine;
use vampirc_uci::UciMessage;

fn main() {
    env_logger::init();
    let (tx, rx) = std::sync::mpsc::channel();
    // Spawn worker thread
    std::thread::spawn(move || {
        let mut engine = Engine::init();
        loop {
            let cmd = rx.recv().unwrap();
            engine.execute_cmd(cmd);
        }
    });
    // Main thread receives and parses stdin and handles some commands
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let message = uci::MessageHandler::parse(input);
        match message {
            uci::MessageHandler::Main(cmd) => {
                match cmd {
                    UciMessage::Quit => break,
                    _ => (),
                };
            }
            uci::MessageHandler::Worker(cmd) => {
                tx.send(cmd).unwrap();
            }
        }
    }
}
