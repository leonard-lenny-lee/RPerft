/// Command Parser and Executor

use super::*;
use state::State;
use regex::Regex;
use std::collections::HashMap;

macro_rules! to_lower {
    ($token: ident) => {
        let binding = $token.to_lowercase();
        let $token = binding.as_str();
    };
}

struct CommandConfig {
    token: &'static str,
    tokens_required: Requires,
    parent_command: Option<CommandType>
}

lazy_static! {
    // Regex patterns for token validations
    static ref MOVE_TOKEN: Regex = Regex::new("([a-h][1-8]){2}[rnbq]?").unwrap();
    static ref ALGB_TOKEN: Regex = Regex::new("[a-h][1-8]").unwrap();

    static ref COMMAND_CONFIGS: HashMap<CommandType, CommandConfig> = {
        HashMap::from([
            (CommandType::Root(Root::Position), CommandConfig {
                token: "position",
                tokens_required: Requires::SubCmd,
                parent_command: None
            }),
            (CommandType::Root(Root::Quit), CommandConfig {
                token: "quit",
                tokens_required: Requires::None,
                parent_command: None
            }),
            (CommandType::Root(Root::Go), CommandConfig {
                token: "go",
                tokens_required: Requires::SubCmd,
                parent_command: None
            }),
            (CommandType::Root(Root::SetOption), CommandConfig {
                token: "setoption",
                tokens_required: Requires::Args(4, 255),
                parent_command: None
            }),
            (CommandType::Leaf(Leaf::Fen), CommandConfig {
                token: "fen",
                tokens_required: Requires::Args(1, 255),
                parent_command: Some(CommandType::Root(Root::Position))
            }),
            (CommandType::Leaf(Leaf::StartPos), CommandConfig {
                token: "startpos",
                tokens_required: Requires::Args(0, 255),
                parent_command: Some(CommandType::Root(Root::Position))
            }),
            (CommandType::Leaf(Leaf::Perft), CommandConfig {
                token: "perft",
                tokens_required: Requires::Args(1, 1),
                parent_command: Some(CommandType::Root(Root::Go))
            }),
            (CommandType::Leaf(Leaf::Display), CommandConfig {
                token: "display",
                tokens_required: Requires::None,
                parent_command: Some(CommandType::Root(Root::Position))
            }),
            (CommandType::Leaf(Leaf::Move), CommandConfig {
                token: "move",
                tokens_required: Requires::Args(1, 255),
                parent_command: Some(CommandType::Root(Root::Position))
            }),
            (CommandType::Leaf(Leaf::Undo), CommandConfig {
                token: "undo",
                tokens_required: Requires::Args(0, 1),
                parent_command: Some(CommandType::Root(Root::Position))
            })
        ])
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandType {
    Root(Root),
    Branch(Branch),
    Leaf(Leaf)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Root {
    Position,
    Quit,
    Go,
    SetOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Branch {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Leaf {
    Fen,
    StartPos,
    Perft,
    Display,
    Move,
    Undo,
}

impl CommandType {

    fn parse(token: &str, level: u8) -> Result<Self, ParseError> {
        to_lower!(token);
        for (key, config) in COMMAND_CONFIGS.iter() {
            if token != config.token {
                continue;
            }
            // Check that for base commands, there is no parent command
            if level == 0 && !matches!(config.parent_command, None) {
                return Err(ParseError::InvalidCommand(token.to_string()))
            }
            return Ok(*key)
        }
        return Err(ParseError::UnrecognisedTokens(token.to_string()))
    }

    fn as_str(&self) -> &str {
        COMMAND_CONFIGS.get(self).unwrap().token
    }

}

enum Requires {
    SubCmd,
    Args(u8, u8), // Min, max # of arguments required
    None,
}

#[derive(Debug)]
pub enum ParseError {
    NullInput,
    InvalidCommand(String),
    InvalidSubCommand(CommandType, CommandType),
    MissingTokens(CommandType),
    UnrecognisedTokens(String),
    InvalidFen(String),
    MissingArguments(CommandType, u8, u8),
    ExcessArguments(CommandType, u8, u8),
}

impl ParseError {

    pub fn warn(&self) {
        let msg = match self {
            Self::NullInput => "No input detected".to_string(),
            Self::InvalidCommand(msg) => format!("Invalid command \"{msg}\""),
            Self::InvalidSubCommand(cmd, subcmd) => {
                let (cmd, subcmd) = (cmd.as_str(), subcmd.as_str());
                format!("\"{subcmd}\" is an invalid subcommand for \"{cmd}\"")
            }
            Self::MissingTokens(token) => {
                let token = token.as_str();
                format!("Additional tokens required for \"{token}\"")
            },
            Self::UnrecognisedTokens(tokens) => format!("Unrecognised token(s): {tokens}"),
            Self::InvalidFen(msg) => format!("Invalid FEN string: {msg}"),
            Self::MissingArguments(token, min, n_tokens) => {
                let token = token.as_str();
                format!("Missing argument(s) for \"{token}\": {min} required, {n_tokens} provided")
            },
            Self::ExcessArguments(token, max, n_tokens) => {
                let token = token.as_str();
                format!("Too many arguments for \"{token}\": {max} allowed, {n_tokens} provided")
            }
        };
        println!("[ERROR] - Could not parse command: {msg}");
    }

}

#[derive(Debug)]
pub enum ExecutionError {
    ParseFenError(String),
    ParseAlgebraicError(String),
    InvalidMoveError(String, String),
    NullPromotionError(String, String)
}

impl ExecutionError {
    pub fn warn(&self) {
        let msg = match self {
            Self::ParseFenError(msg) => 
                format!("Could not parse FEN: {msg}"),
            Self::ParseAlgebraicError(msg) => 
                format!("Could not parse move: {msg}"),
            Self::InvalidMoveError(mv, fen) => 
                format!("Invalid move \"{mv}\" in the position {fen}"),
            Self::NullPromotionError(mv, fen) =>
                format!("Missing promotion specifier for \"{mv}\" in the position {fen}")
        };
        println!("[ERROR] - {msg}")
    }
}

pub struct Command {
    pub cmd: CommandType,
    subcmd: Option<Box<Command>>,
    args: Option<Vec<String>>,
}

impl Command {

    /// Tokenize and parse the input string into a Command struct
    pub fn parse(input: String) -> Result<Self, ParseError> {
        // Tokenize
        let input = input.trim();
        if input.len() == 0 {
            return Err(ParseError::NullInput)
        }
        let tokens: Vec<&str> = input.split_whitespace().collect();
        return Self::parse_tokens(&tokens, 0)
    }

    fn parse_tokens(tokens: &Vec<&str>, level: u8) -> Result<Self, ParseError> {
        let cmd = CommandType::parse(tokens[0], level)?;
        let args = tokens[1..].to_vec();
        let n_tokens = args.len() as u8;
        // Check extra token requirements and build command struct accordingly
        match COMMAND_CONFIGS.get(&cmd).unwrap().tokens_required {
            Requires::SubCmd => {
                if n_tokens == 0 {
                    return Err(ParseError::MissingTokens(cmd))
                }
                let subcmd = Self::parse_tokens(&args, level+1)?;
                Self::check_subcommand(&cmd, &subcmd.cmd)?;
                Ok(Self {cmd, subcmd: Some(Box::new(subcmd)), args: None})
            },
            Requires::Args(min, max) => {
                if n_tokens < min {
                    return Err(ParseError::MissingArguments(cmd, min, n_tokens));
                }
                if n_tokens > max {
                    return Err(ParseError::ExcessArguments(cmd, max, n_tokens));
                }
                Self::check_arguments(&cmd, &args)?;
                let args = args
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                Ok(Self {cmd, subcmd: None, args: Some(args)})
            },
            Requires::None => {
                if n_tokens > 0 {
                    return Err(ParseError::ExcessArguments(cmd, 0, n_tokens))
                }
                Ok(Self {cmd, subcmd: None, args: None})
            }
        }
    }

    /// Check that the subcommand provided is a valid option for the command
    /// * only should be invoked for commands requiring subcommands
    fn check_subcommand(
        cmd: &CommandType, subcmd: &CommandType
    ) -> Result<(), ParseError> {
        if let Some(config) = COMMAND_CONFIGS.get(subcmd) {
            let valid = match config.parent_command {
                Some(parent_command) => *cmd == parent_command,
                None => false
            };
            if !valid {
                return Err(ParseError::InvalidSubCommand(*cmd, *subcmd));
            }
            return Ok(())
        } else {
            panic!() // DEBUGGING
        }
    }

    /// Check that the arguments provided conform to the format expected
    /// * only should be invoked for commands requiring arguments
    fn check_arguments(
        cmd: &CommandType, args: &Vec<&str>
    ) -> Result<(), ParseError> {
        use Leaf::*;
        if let CommandType::Leaf(cmd) = cmd {
            match cmd {
                Fen => args_check::fen_tokens(args)?,
                StartPos | Move => args_check::move_tokens(args)?,
                Undo => args_check::undo_token(args)?,
                Perft => args_check::perft_token(args)?,
                Display => (),
            }
        } else {
            println!("WARNING! Attempted argument parsing of non-leaf command")
        }
        Ok(())
    }

    pub fn execute(&self, state: &mut State) -> Result<(), ExecutionError> {
        // Only execute command if it's a leaf command i.e. no sub-command
        match &self.subcmd {
            Some(subcmd) => subcmd.execute(state),
            None => self.execute_cmd(state)
        }
    }

    fn execute_cmd(&self, state: &mut State) -> Result<(), ExecutionError> {
        if let CommandType::Leaf(cmd) = self.cmd {
            match cmd {
                Leaf::Perft => {
                    if let Some(token) = &self.args {
                        execute::perft(state, token[0].to_string())?
                    }
                },
                Leaf::Display => {
                    println!("{}", state.position.data.to_string());
                },
                Leaf::Fen => {
                    if let Some(args) = &self.args {
                        execute::fen(state, args)?
                    }
                },
                Leaf::StartPos => {
                    if let Some(args) = &self.args {
                        execute::startpos(state, args)?
                    }
                },
                Leaf::Move => {
                    if let Some(args) = &self.args {
                        execute::moves(state, args)?
                    }
                },
                Leaf::Undo => {
                    if let Some(args) = &self.args {
                        execute::undo(state, args)?
                    }
                },
            }
        } else {
            println!("WARNING! Attempted execution of non-leaf command")
        }
        Ok(())
    }

}

mod args_check {

    use super::*;

    pub fn fen_tokens(args: &Vec<&str>) -> Result<(), ParseError> {
        if args.len() < 6 {
            return Err(ParseError::InvalidFen("Insufficient number of tokens".to_string()))
        };
        self::fen_board_token(args[0])?;
        self::wtm_token(args[1])?;
        self::castle_token(args[2])?;
        self::ep_token(args[3])?;
        self::clock_token(args[4])?;
        self::clock_token(args[5])?;
        // Any extra tokens are move specifier tokens
        self::move_tokens(&args[6..].to_vec())?;
        Ok(())
    }

    fn fen_board_token(token: &str) -> Result<(), ParseError> {
        // Check that only valid characters are in the token
        const VALID_CHARS: [char; 21] = [
            'P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k',
            '/', '1', '2', '3', '4', '5', '6', '7', '8'
        ];
        if !token.chars().all(|c| VALID_CHARS.contains(&c)) {
            return Err(ParseError::InvalidFen(format!("Invalid board token \"{token}\"")))
        }
        let (mut n_delimiters, mut n_squares) = (0, 0);
        for c in token.chars() {
            if c.is_alphabetic() {
                n_squares += 1;
            } else if c.is_numeric() {
                let n = c.to_digit(10).unwrap();
                n_squares += n;
            } else {
                n_delimiters += 1
            }
        }
        if n_delimiters != 7 || n_squares != 64 {
            return Err(ParseError::InvalidFen(format!("Invalid board token \"{token}\"")))
        }
        Ok(())
    }

    fn wtm_token(token: &str) -> Result<(), ParseError> {
        if token == "w" || token == "b" {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!("Invalid w.t.m. token \"{token}\"")))
        }
    }

    fn castle_token(token: &str) -> Result<(), ParseError> {
        const VALID_CHARS: [char; 4] = ['K', 'k', 'Q', 'q'];
        if (token.chars().all(|c| VALID_CHARS.contains(&c) && token.len() <= 4)) || token == "-" {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!("Invalid castle token \"{token}\"")))
        }
    }

    fn ep_token(token: &str) -> Result<(), ParseError> {
        if ALGB_TOKEN.is_match(token) || token == "-" {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!("Invalid e.p. token \"{token}\"")))
        }
    }

    fn clock_token(token: &str) -> Result<(), ParseError> {
        if let Err(_) = token.parse::<u32>() {
            Err(ParseError::InvalidFen(format!("Invalid clock token \"{token}\"")))
        } else {
            Ok(())
        }
    }

    pub fn move_tokens(args: &Vec<&str>) -> Result<(), ParseError> {
        let mut invalid_tokens = Vec::new();
        for token in args {
            if !MOVE_TOKEN.is_match(token) {
                invalid_tokens.push(*token)
            }
        }
        if invalid_tokens.len() >= 1 {
            return Err(ParseError::UnrecognisedTokens(invalid_tokens.join(" ")))
        }
        return Ok(())
    }

    pub fn perft_token(token: &Vec<&str>) -> Result<(), ParseError> {
        if !token[0].chars().all(|c| c.is_numeric()) && token[0] != "bench"{
            return Err(ParseError::UnrecognisedTokens(token[0].to_string()))
        }
        Ok(())
    }

    pub fn undo_token(token: &Vec<&str>) -> Result<(), ParseError> {
        if token.len() == 1 {
            if let Err(_) = token[0].parse::<u32>() {
                return Err(ParseError::UnrecognisedTokens(token[0].to_string()))
            }
        }
        Ok(())
    }

}

mod execute {

    use super::*;

    pub fn perft(state: &mut State, arg: String) -> Result<(), ExecutionError> {
        let token = arg.parse::<i8>();
        match token {
            Ok(depth) => {
                search::perft::perft_divided(&state.position, depth, &state.config);
            },
            Err(_) => search::perft::run_perft_bench(),
        };
        Ok(())
    }

    pub fn fen(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        let fen = args[0..6].join(" ");
        let moves = args[6..].to_vec();
        state.position_history.push(state.position.clone());
        state.position = position::Position::from_fen(fen)?;
        self::moves(state, &moves)
    }

    pub fn startpos(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        state.position_history.push(state.position.clone());
        state.position = position::Position::from_fen(common::DEFAULT_FEN.to_string())?;
        self::moves(state, args)
    }

    pub fn moves(state: &mut State, moves: &Vec<String>) -> Result<(), ExecutionError> {
        for move_token in moves {
            let move_list = movegen::find_moves(&state.position);
            let src = BB::from_algebraic(&move_token[0..2])?;
            let target = BB::from_algebraic(&move_token[2..4])?;
            let mut matches = Vec::new();
            for mv in move_list.iter() {
                if mv.src() == src && mv.target() == target {
                    if move_token.len() == 4 {
                        matches.push(mv);
                        continue;
                    }
                    let promotion_piece = match &move_token[4..=4] {
                        "r" => Piece::Rook.value(),
                        "n" => Piece::Knight.value(),
                        "b" => Piece::Bishop.value(),
                        "q" => Piece::Queen.value(),
                        _ => return Err(ExecutionError::InvalidMoveError(
                            move_token.to_string(), state.position.data.fen()
                        ))
                    };
                    if mv.is_promotion() && promotion_piece == mv.promotion_piece() {
                        matches.push(mv)
                    }
                }
            }
            match matches.len().cmp(&1) {
                std::cmp::Ordering::Less => {
                    // Cannot find the specified move in the position
                    return Err(ExecutionError::InvalidMoveError(
                        move_token.to_string(), state.position.data.fen()
                    ))
                },
                std::cmp::Ordering::Equal => {
                    // Execute the move
                    state.position_history.push(state.position.clone());
                    state.position = makemove::make_move(&state.position, &matches[0])
                },
                std::cmp::Ordering::Greater => {
                    // If there is more than 1 move, a promotion specifier was
                    // misiing
                    return Err(ExecutionError::NullPromotionError(
                        move_token.to_string(), state.position.data.fen()
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn undo(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        let n = if args.len() == 0 {1} else {args[0].parse::<u32>().unwrap()};
        let n = std::cmp::max(n, state.position_history.len() as u32);
        for _ in 0..n {
            if let Some(pos) = state.position_history.pop() {
                state.position = pos;
            }
        };
        Ok(())
    }

}