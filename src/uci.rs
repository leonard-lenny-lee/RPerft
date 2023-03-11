/// UCI protocol interface using the vampirc_uci crate.
use super::*;

use regex::Regex;
use vampirc_uci::{
    parse_one, CommunicationDirection, Serializable, UciFen, UciMessage, UciMove, UciSearchControl,
    UciTimeControl,
};

use engine::{Engine, EngineMode};
use position::Position;

// The message be handled by the main thread or the worker thread
pub enum MessageHandler {
    Main(UciMessage),
    Worker(UciMessage),
}

impl MessageHandler {
    // Parse input from the stdin into a UciMessage and assign messages to the
    // correct thread for execution
    pub fn parse(input: String) -> Self {
        let message = parse_one(input.as_str());
        if matches!(message, UciMessage::Quit) {
            return Self::Main(message);
        }
        return Self::Worker(message);
    }
}

impl Engine {
    /// Match UciMessage and execute command instructed
    pub fn execute_cmd(&mut self, cmd: UciMessage) {
        match cmd {
            UciMessage::Uci => self.exec_uci(),
            UciMessage::Debug(on) => self.debug = on,
            UciMessage::IsReady => self.exec_isready(),
            // Registration not required, always ignore this command
            UciMessage::Register { .. } => (),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => self.exec_position(startpos, fen, moves),
            UciMessage::SetOption { .. } => todo!(),
            UciMessage::UciNewGame => self.exec_ucinewgame(),
            UciMessage::Stop => todo!(),
            UciMessage::PonderHit => todo!(),
            UciMessage::Quit => (),
            UciMessage::Go {
                time_control,
                search_control,
            } => self.exec_go(time_control, search_control),
            UciMessage::Unknown(msg, _) => self.exec_unknown(msg),
            _ => assert!(matches!(
                cmd.direction(),
                CommunicationDirection::GuiToEngine
            )),
        }
    }

    // Command executor methods

    fn exec_uci(&mut self) {
        self.mode = EngineMode::Uci;
        println!("{}", UciMessage::id_name(ENGINE_NAME).serialize());
        println!("{}", UciMessage::id_author(AUTHOR_NAME).serialize());
        // TODO Send configurable option data via "option" commands
        println!("{}", UciMessage::UciOk.serialize());
    }

    fn exec_isready(&self) {
        // Respond with "readyok"
        println!("{}", UciMessage::ReadyOk.serialize());
    }

    fn exec_ucinewgame(&mut self) {
        self.hash_table.clear();
    }

    fn exec_position(&mut self, startpos: bool, fen: Option<UciFen>, moves: Vec<UciMove>) {
        if startpos {
            self.cur_pos = Position::new_starting();
        }
        if let Some(f) = fen {
            // Attempt to parse and load the fen string into the position
            match Position::from_fen(f.to_string()) {
                Ok(p) => self.cur_pos = p,
                Err(e) => match self.mode {
                    EngineMode::Uci => (),
                    EngineMode::User => e.warn(),
                },
            }
        }
        // Sequentially apply the moves specified to the position
        for ucimove in moves {
            let mut valid = false;
            let movelist = movegen::find_moves(&self.cur_pos);
            // Scan movelist for ucimove specified move
            for mv in movelist.iter() {
                // Found move so play it and mark move as valid
                if format!("{ucimove}") == mv.to_algebraic() {
                    self.cur_pos = makemove::make_move(&self.cur_pos, mv);
                    valid = true;
                    break;
                }
            }
            // Warn user of invalid move, ignore in UCI mode
            if !valid && matches!(self.mode, EngineMode::User) {
                RuntimeError::InvalidMoveError {
                    mv: format!("{ucimove}"),
                    fen: self.cur_pos.data.fen(),
                }
                .warn()
            }
        }
    }

    fn exec_go(
        &self,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) {
        if let Some(tc) = time_control {
            // TODO Awaiting implementation of iterative deepening
            match tc {
                UciTimeControl::Ponder => todo!(),
                UciTimeControl::Infinite => todo!(),
                UciTimeControl::TimeLeft { .. } => todo!(),
                UciTimeControl::MoveTime(_) => todo!(),
            }
        }
        if let Some(_sc) = search_control {
            // TODO Implement search
        }
    }

    fn exec_unknown(&mut self, msg: String) {
        // Ignore unknown messages in UCI mode
        if !matches!(self.mode, EngineMode::User) {
            return;
        }
        // Attempt to parse custom inputs
        lazy_static! {
            static ref PERFT_D: Regex = Regex::new(r"^go perft \d+$").unwrap();
        }
        let msg = msg.trim();
        if msg == "go perft bench" {
            search::perft::run_perft_suite(self.num_threads, self.table_size_bytes);
            return;
        }
        if msg == "d" || msg == "display" {
            print!("{}", self.cur_pos.to_string());
            return;
        }
        if PERFT_D.is_match(msg) {
            let mut m: Vec<&str> = msg.split(" ").collect();
            let depth = m
                .pop()
                .expect("regex matched 3 tokens")
                .parse::<u8>()
                .expect("regex matched last token as numerical");
            search::perft::perft(
                &self.cur_pos,
                depth,
                self.num_threads,
                self.table_size_bytes,
                true,
            );
            return;
        }
        // Else unrecognised command
        RuntimeError::StdinParseError(msg.to_string()).warn();
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StdinParseError(String),
    ParseFenError(String),
    ParseAlgebraicError(String),
    InvalidMoveError { mv: String, fen: String },
}

impl RuntimeError {
    pub fn warn(&self) {
        let msg = match self {
            Self::StdinParseError(msg) => {
                format!("Could not parse command: {msg}")
            }
            Self::ParseFenError(msg) => {
                format!("Could not parse FEN: {msg}")
            }
            Self::ParseAlgebraicError(msg) => {
                format!("Could not parse move: {msg}")
            }
            Self::InvalidMoveError { mv, fen } => {
                format!("Invalid move \"{mv}\" in the position {fen}")
            }
        };
        log::error!("{msg}")
    }
}
