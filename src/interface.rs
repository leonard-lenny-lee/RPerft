/// Command Parser and Executor

use super::*;
use state::State;
use regex::Regex;

macro_rules! to_lower {
    ($token: ident) => {
        let binding = $token.to_lowercase();
        let $token = binding.as_str();
    };
}

lazy_static! {
    static ref MOVE_TOKEN: Regex = Regex::new("([a-h][1-8]){2}[rnbq]?").unwrap();
    static ref ALGB_TOKEN: Regex = Regex::new("[a-h][1-8]").unwrap();
}

#[derive(Debug, Clone, Copy)]
pub enum CommandToken {
    // Level 0 Commands
    Position,
    Quit,
    Go,
    SetOption,
    // Level 1 Commands
    Fen,
    StartPos,
    Perft,
    Show,
}

impl CommandToken {

    fn parse(token: &str, level: u8) -> Result<Self, ParseErr> {
        to_lower!(token);
        match level {
            0 => match token {
                "position" => Ok(Self::Position),
                "quit" => Ok(Self::Quit),
                "go" => Ok(Self::Go),
                "setoption" => Ok(Self::SetOption),
                _ => Err(ParseErr::InvalidCommand(token.to_string()))
            },
            1 => match token {
                "fen" => Ok(Self::Fen),
                "startpos" => Ok(Self::StartPos),
                "perft" => Ok(Self::Perft),
                "show" => Ok(Self::Show),
                _ => Err(ParseErr::InvalidCommand(token.to_string()))
            },
            _ => Err(ParseErr::UnrecognisedTokens(token.to_string()))
        }  
    }

    /// Mappings to get the extra token requirements for the command
    fn get_token_requirements(&self) -> Requires {
        match self {
            Self::Position => Requires::SubCmd,
            Self::Quit => Requires::None,
            Self::Go => Requires::SubCmd,
            Self::SetOption => Requires::Args(4, 255),
            Self::Fen => Requires::Args(1, 255),
            Self::StartPos => Requires::Args(0, 255),
            Self::Perft => Requires::Args(1, 1),
            Self::Show => Requires::None,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Position => "position",
            Self::Quit => "quit",
            Self::Go => "go",
            Self::SetOption => "setoption",
            Self::Fen => "fen",
            Self::StartPos => "startpos",
            Self::Perft => "perft",
            Self::Show => "show"
        }
    }

}

enum Requires {
    SubCmd,
    Args(u8, u8), // Min, max # of arguments required
    None,
}

#[derive(Debug)]
pub enum ParseErr {
    NullInput,
    InvalidCommand(String),
    InvalidSubCommand(CommandToken, CommandToken),
    MissingTokens(CommandToken),
    UnrecognisedTokens(String),
    InvalidFen(String),
    MissingArguments(CommandToken, u8, u8),
    ExcessArguments(CommandToken, u8, u8),
}

impl ParseErr {

    pub fn warn(&self) {
        // TODO Write warning messages
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
        println!("WARNING! Error parsing command: {msg}");
    }

}

#[derive(Debug)]
pub enum ExecutionErr {
    FenErr(String)
}

impl ExecutionErr {
    pub fn warn(&self) {
        // TODO Write warning messages
    }
}

pub struct Command {
    pub cmd: CommandToken,
    subcmd: Option<Box<Command>>,
    args: Option<Vec<String>>,
}

impl Command {

    /// Tokenize and parse the input string into a Command struct
    pub fn parse(input: String) -> Result<Self, ParseErr> {
        // Tokenize
        let input = input.trim();
        if input.len() == 0 {
            return Err(ParseErr::NullInput)
        }
        let tokens: Vec<&str> = input.split_whitespace().collect();
        return Self::parse_tokens(&tokens, 0)
    }

    fn parse_tokens(tokens: &Vec<&str>, level: u8) -> Result<Self, ParseErr> {
        let cmd = CommandToken::parse(tokens[0], level)?;
        let args = tokens[1..].to_vec();
        let n_tokens = args.len() as u8;
        // Check extra token requirements and build command struct accordingly
        match cmd.get_token_requirements() {
            Requires::SubCmd => {
                if n_tokens == 0 {
                    return Err(ParseErr::MissingTokens(cmd))
                }
                let subcmd = Self::parse_tokens(&args, level+1)?;
                Self::check_subcommand(&cmd, &subcmd.cmd)?;
                Ok(Self {cmd, subcmd: Some(Box::new(subcmd)), args: None})
            },
            Requires::Args(min, max) => {
                if n_tokens < min {
                    return Err(ParseErr::MissingArguments(cmd, min, n_tokens));
                }
                if n_tokens > max {
                    return Err(ParseErr::ExcessArguments(cmd, max, n_tokens));
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
                    return Err(ParseErr::ExcessArguments(cmd, 0, n_tokens))
                }
                Ok(Self {cmd, subcmd: None, args: None})
            }
        }
    }

    /// Check that the subcommand provided is a valid option for the command
    /// * only should be invoked for commands requiring subcommands
    fn check_subcommand(
        cmd: &CommandToken, subcmd: &CommandToken
    ) -> Result<(), ParseErr> {
        use CommandToken::*;
        let valid = match cmd {
            Position => matches!(subcmd, Fen | StartPos | Show),
            Go => matches!(subcmd, Perft),
            _ => panic!("INVALID SUBCOMMAND CHECK LOGIC") // For debugging
        };
        if !valid {
            return Err(ParseErr::InvalidSubCommand(*cmd, *subcmd))
        }
        Ok(())
    }

    /// Check that the arguments provided conform to the format expected
    /// * only should be invoked for commands requiring arguments
    fn check_arguments(
        cmd: &CommandToken, args: &Vec<&str>
    ) -> Result<(), ParseErr> {
        use CommandToken::*;
        match cmd {
            Fen => Command::check_fen_tokens(args),
            StartPos => Command::check_move_tokens(args),
            Perft => Command::check_perft_token(args),
            _ => panic!("INVALID ARGUMENT CHECK LOGIC") // For debugging
        }
    }

    fn check_fen_tokens(args: &Vec<&str>) -> Result<(), ParseErr> {
        if args.len() < 6 {
            return Err(ParseErr::InvalidFen("Insufficient number of tokens".to_string()))
        };
        Command::check_fen_board_token(args[0])?;
        Command::check_wtm_token(args[1])?;
        Command::check_castle_token(args[2])?;
        Command::check_ep_token(args[3])?;
        Command::check_clock_token(args[4])?;
        Command::check_clock_token(args[5])?;
        // Any extra tokens are move specifier tokens
        Command::check_move_tokens(&args[6..].to_vec())?;
        Ok(())
    }

    fn check_fen_board_token(token: &str) -> Result<(), ParseErr> {
        // Check that only valid characters are in the token
        const VALID_CHARS: [char; 21] = [
            'P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k',
            '/', '1', '2', '3', '4', '5', '6', '7', '8'
        ];
        if !token.chars().all(|c| VALID_CHARS.contains(&c)) {
            return Err(ParseErr::InvalidFen(format!("Invalid board token \"{token}\"")))
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
            return Err(ParseErr::InvalidFen(format!("Invalid board token \"{token}\"")))
        }
        Ok(())
    }

    fn check_wtm_token(token: &str) -> Result<(), ParseErr> {
        if token == "w" || token == "b" {
            Ok(())
        } else {
            Err(ParseErr::InvalidFen(format!("Invalid w.t.m. token \"{token}\"")))
        }
    }

    fn check_castle_token(token: &str) -> Result<(), ParseErr> {
        const VALID_CHARS: [char; 4] = ['K', 'k', 'Q', 'q'];
        if (token.chars().all(|c| VALID_CHARS.contains(&c) && token.len() <= 4)) || token == "-" {
            Ok(())
        } else {
            Err(ParseErr::InvalidFen(format!("Invalid castle token \"{token}\"")))
        }
    }

    fn check_ep_token(token: &str) -> Result<(), ParseErr> {
        if ALGB_TOKEN.is_match(token) || token == "-" {
            Ok(())
        } else {
            Err(ParseErr::InvalidFen(format!("Invalid e.p. token \"{token}\"")))
        }
    }

    fn check_move_tokens(args: &Vec<&str>) -> Result<(), ParseErr> {
        let mut invalid_tokens = Vec::new();
        for token in args {
            if !MOVE_TOKEN.is_match(token) {
                invalid_tokens.push(*token)
            }
        }
        if invalid_tokens.len() >= 1 {
            return Err(ParseErr::UnrecognisedTokens(invalid_tokens.join(" ")))
        }
        return Ok(())
    }

    fn check_clock_token(token: &str) -> Result<(), ParseErr> {
        if let Err(_) = token.parse::<u32>() {
            Err(ParseErr::InvalidFen(format!("Invalid clock token \"{token}\"")))
        } else {
            Ok(())
        }
    }

    fn check_perft_token(token: &Vec<&str>) -> Result<(), ParseErr> {
        if !token[0].chars().all(|c| c.is_numeric()) && token[0] != "bench"{
            return Err(ParseErr::UnrecognisedTokens(token[0].to_string()))
        }
        Ok(())
    }

    pub fn execute(&self, state: &mut State) -> Result<(), ExecutionErr> {
        // Only execute command if it's a leaf command i.e. no sub-command
        match &self.subcmd {
            Some(subcmd) => subcmd.execute(state),
            None => self.execute_cmd(state)
        }
    }

    fn execute_cmd(&self, state: &mut State) -> Result<(), ExecutionErr> {
        // TODO Finish
        match self.cmd {
            CommandToken::Perft => {
                if let Some(token) = &self.args {
                    let depth = token[0].parse::<i8>().unwrap();
                    search::perft::perft_divided(&state.position, depth, &state.config);
                }
            },
            CommandToken::Show => {
                println!("{}", state.position.data.board());
            },
            CommandToken::Fen => {
                if let Some(args) = &self.args {
                    let fen = args[0..6].join(" ");
                    state.position_history.push(state.position.clone());
                    state.position = position::Position::from_fen(fen)?
                }
            }
            _ => ()
        }
        Ok(())
    }

}
