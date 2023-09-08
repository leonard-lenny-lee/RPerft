/// UCI protocol interface using the v_uci crate.
use super::*;

use regex::Regex;
use v_uci::{
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
        Self::Worker(message)
    }
}

impl Engine {
    /// Match UciMessage and execute command instructed
    pub fn execute_cmd(&mut self, cmd: UciMessage) {
        let r = match cmd {
            UciMessage::Uci => self.execute_uci(),
            UciMessage::Debug(_on) => Ok(()), // TODO delegate to setoption
            UciMessage::IsReady => self.execute_isready(),
            // Registration not required, always ignore this command
            UciMessage::Register { .. } => Ok(()),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => self.execute_position(startpos, fen, moves),
            UciMessage::SetOption { .. } => todo!(),
            UciMessage::UciNewGame => self.execute_ucinewgame(),
            UciMessage::Stop => todo!(),
            UciMessage::PonderHit => todo!(),
            UciMessage::Quit => Ok(()),
            UciMessage::Go {
                time_control,
                search_control,
            } => self.execute_go(time_control, search_control),
            UciMessage::Unknown(msg, _) => self.execute_unknown(msg),
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

    /// Handle "uci" commands
    fn execute_uci(&mut self) -> Result<(), RuntimeError> {
        self.mode = EngineMode::Uci;
        let engine_name = UciMessage::id_name(constants::ENGINE_NAME).serialize();
        let author_name = UciMessage::id_author(constants::AUTHOR_NAME).serialize();

        println!("{engine_name}\n{author_name}",);

        // TODO Send configurable option data via "option" commands
        println!(
            "{}",
            UciMessage::Option(UciOptionConfig::Spin {
                name: "Hash".to_string(),
                default: Some(32),
                min: Some(16),
                max: None,
            })
        );

        println!("{}", UciMessage::UciOk);
        Ok(())
    }

    /// Execute "isready" commands
    fn execute_isready(&self) -> Result<(), RuntimeError> {
        // Respond with "readyok"
        println!("{}", UciMessage::ReadyOk);
        Ok(())
    }

    /// Future moves are from a different game
    fn execute_ucinewgame(&mut self) -> Result<(), RuntimeError> {
        self.hash_table.clear();
        Ok(())
    }

    // Modify current game position in response to "position" commands
    fn execute_position(
        &mut self,
        startpos: bool,
        fen: Option<UciFen>,
        moves: Vec<UciMove>,
    ) -> Result<(), RuntimeError> {
        if startpos {
            self.current_position = Position::new_starting_position();
        }

        // Attempt to parse and load the fen string into the position
        if let Some(f) = fen {
            self.current_position = Position::from_fen(&f.to_string()[..])?
        }

        // Sequentially apply the moves specified to the position
        for ucimove in moves {
            let mut movelist = movelist::UnorderedList::new();
            movegen::generate_all(&self.current_position, &mut movelist);
            let mv_algebraic = format!("{ucimove}");

            if let Some(mv) = movelist.find(mv_algebraic) {
                self.current_position.make_move(&mv);
                continue;
            }

            return Err(RuntimeError::InvalidMoveError {
                move_algebraic: format!("{ucimove}"),
                fen_str: self.current_position.to_fen(),
            });
        }

        return Ok(());
    }

    // Execute search and calculation commands
    fn execute_go(
        &mut self,
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

        if let Some(sc) = search_control {
            if let Some(depth) = sc.depth {
                search::search(&mut self.current_position, depth, &mut self.hash_table)
            }
        }

        return Ok(());
    }

    // Check for command patterns not in the UCI protocol, for human-input
    fn execute_unknown(&mut self, msg: String) -> Result<(), RuntimeError> {
        // Ignore unknown messages in UCI mode
        if !matches!(self.mode, EngineMode::User) {
            return Ok(());
        }
        // Attempt to parse custom inputs
        lazy_static! {
            static ref PERFT_D: Regex = Regex::new(r"^go perft \d+$").unwrap();
        }

        let message = msg.trim();

        // Perft test runner suite
        if message == "go perft bench" {
            search::perft::run_perft_benchmark_suite(self.num_threads, self.table_size_bytes);
            return Ok(());
        }

        // Display current position board
        if message == "d" || message == "display" {
            print!("{}", self.current_position.to_string());
            return Ok(());
        }

        // Perft runner on current position
        if PERFT_D.is_match(message) {
            let mut m: Vec<&str> = message.split(" ").collect();

            let depth = m
                .pop()
                .expect("regex matched 3 tokens")
                .parse::<u8>()
                .expect("regex matched last token as numerical");

            search::perft::perft(
                &self.current_position,
                depth,
                self.num_threads,
                self.table_size_bytes,
                true,
            );
            return Ok(());
        }

        // Else, unrecognised command, raise runtime error
        return Err(RuntimeError::ParseStdinError {
            str: message.to_string(),
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
    AlgebraicParseError(String),
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
            Self::AlgebraicParseError(msg) => {
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
