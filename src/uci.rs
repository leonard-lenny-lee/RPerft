/// UCI protocol interface using the vampirc_uci crate.
use super::*;

use regex::Regex;
use vampirc_uci::{
    parse_one, CommunicationDirection, Serializable, UciFen, UciMessage, UciMove, UciOptionConfig,
    UciSearchControl, UciTimeControl,
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
        let r = match cmd {
            UciMessage::Uci => self.exec_uci(),
            UciMessage::Debug(_on) => Ok(()), // TODO delegate to setoption
            UciMessage::IsReady => self.exec_isready(),
            // Registration not required, always ignore this command
            UciMessage::Register { .. } => Ok(()),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => self.exec_position(startpos, fen, moves),
            UciMessage::SetOption { .. } => todo!(),
            UciMessage::UciNewGame => self.exec_ucinewgame(),
            UciMessage::Stop => todo!(),
            UciMessage::PonderHit => todo!(),
            UciMessage::Quit => Ok(()),
            UciMessage::Go {
                time_control,
                search_control,
            } => self.exec_go(time_control, search_control),
            UciMessage::Unknown(msg, _) => self.exec_unknown(msg),
            _ => match cmd.direction() {
                CommunicationDirection::EngineToGui => {
                    Err(RuntimeError::UciCommError { uci_cmd: cmd })
                }
                CommunicationDirection::GuiToEngine => Err(RuntimeError::ParseStdinError {
                    str: cmd.serialize(),
                }),
            },
        };

        // In user mode, the engine should provide verbose log messages if there
        // are errors with the input. In UCI mode, assume input is correct, ignore
        // any messages that cannot be parsed or executed.
        if let Err(e) = r {
            if matches!(self.mode, EngineMode::User) {
                e.warn()
            }
        }
    }

    /* Command executor methods */

    fn exec_uci(&mut self) -> Result<(), RuntimeError> {
        self.mode = EngineMode::Uci;
        println!("{}", UciMessage::id_name(ENGINE_NAME).serialize());
        println!("{}", UciMessage::id_author(AUTHOR_NAME).serialize());

        // TODO Send configurable option data via "option" commands
        println!(
            "{}",
            UciMessage::Option(UciOptionConfig::Spin {
                name: "Hash".to_string(),
                default: Some(32),
                min: Some(16),
                max: None,
            })
            .serialize()
        );

        println!("{}", UciMessage::UciOk.serialize());
        return Ok(());
    }

    fn exec_isready(&self) -> Result<(), RuntimeError> {
        // Respond with "readyok"
        println!("{}", UciMessage::ReadyOk.serialize());
        return Ok(());
    }

    // Future moves are from a different game
    fn exec_ucinewgame(&mut self) -> Result<(), RuntimeError> {
        self.hash_table.clear();
        return Ok(());
    }

    // Modify current game position
    fn exec_position(
        &mut self,
        startpos: bool,
        fen: Option<UciFen>,
        moves: Vec<UciMove>,
    ) -> Result<(), RuntimeError> {
        if startpos {
            self.cur_pos = Position::new_starting_pos();
        }

        // Attempt to parse and load the fen string into the position
        if let Some(f) = fen {
            self.cur_pos = Position::from_fen(&f.to_string()[..])?
        }

        // Sequentially apply the moves specified to the position
        for ucimove in moves {
            let movelist = movegen::find_moves(&self.cur_pos);
            let mv_algebraic = format!("{ucimove}");

            if let Some(mv) = movelist.find(mv_algebraic) {
                self.cur_pos = self.cur_pos.make_move(&mv);
                continue;
            }

            return Err(RuntimeError::InvalidMoveError {
                move_algebraic: format!("{ucimove}"),
                fen_str: self.cur_pos.to_fen(),
            });
        }

        return Ok(());
    }

    // Execute search and calculation commands
    fn exec_go(
        &self,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    ) -> Result<(), RuntimeError> {
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

        return Ok(());
    }

    // Check for command patterns not in the UCI protocol, for human-input
    fn exec_unknown(&mut self, msg: String) -> Result<(), RuntimeError> {
        // Ignore unknown messages in UCI mode
        if !matches!(self.mode, EngineMode::User) {
            return Ok(());
        }
        // Attempt to parse custom inputs
        lazy_static! {
            static ref PERFT_D: Regex = Regex::new(r"^go perft \d+$").unwrap();
        }

        let msg = msg.trim();

        // Perft test runner suite
        if msg == "go perft bench" {
            search::perft::run_perft_suite(self.num_threads, self.table_size_bytes);
            return Ok(());
        }

        // Display current position board
        if msg == "d" || msg == "display" {
            print!("{}", self.cur_pos.to_string());
            return Ok(());
        }

        // Perft runner on current position
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
            return Ok(());
        }

        // Else, unrecognised command, raise runtime error
        return Err(RuntimeError::ParseStdinError {
            str: msg.to_string(),
        });
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    ParseStdinError {
        str: String,
    },
    UciCommError {
        uci_cmd: UciMessage,
    },
    ParseFenError,
    ParseAlgebraicError(String),
    InvalidMoveError {
        move_algebraic: String,
        fen_str: String,
    },
}

impl RuntimeError {
    pub fn warn(&self) {
        let msg = match self {
            Self::ParseStdinError { str } => {
                format!("Could not parse command: {str}")
            }
            Self::UciCommError { uci_cmd } => {
                format!("UCI command {uci_cmd} in wrong iostream")
            }
            Self::ParseFenError => {
                format!("Could not parse FEN. Check string.")
            }
            Self::ParseAlgebraicError(msg) => {
                format!("Could not parse move: {msg}")
            }
            Self::InvalidMoveError {
                move_algebraic,
                fen_str,
            } => {
                format!("Invalid move \"{move_algebraic}\" in the position {fen_str}")
            }
        };
        log::error!("{msg}")
    }
}
