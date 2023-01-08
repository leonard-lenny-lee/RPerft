/// Command Parser and Executor
use super::*;
use regex::Regex;
use state::State;
use std::collections::HashMap;

macro_rules! to_lower {
    ($token: ident) => {
        let binding = $token.to_lowercase();
        let $token = binding.as_str();
    };
}

// Configuration struct for the COMMAND_CONFIG hashmap
struct CommandConfig {
    token: &'static str,
    tokens_required: Requires,
    parent_command: Command,
    level: u8,
}

lazy_static! {
    // Regex patterns for token validations
    static ref MOVE_TOKEN: Regex = Regex::new("([a-h][1-8]){2}[rnbq]?").unwrap();
    static ref ALGB_TOKEN: Regex = Regex::new("[a-h][1-8]").unwrap();

    // Command-specific requirements and configurations used for parsing and
    // validation
    static ref COMMAND_CONFIGS: HashMap<Command, CommandConfig> = {
        HashMap::from([
            (Command::Root, CommandConfig {
                token: "$ROOT",
                tokens_required: Requires::SubCmd,
                parent_command: Command::Root,
                level: 0
            }),
            (Command::Branch(Branch::Position), CommandConfig {
                token: "position",
                tokens_required: Requires::SubCmd,
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Leaf(Leaf::Quit), CommandConfig {
                token: "quit",
                tokens_required: Requires::None,
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Branch(Branch::Go), CommandConfig {
                token: "go",
                tokens_required: Requires::SubCmd,
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Leaf(Leaf::SetOption), CommandConfig {
                token: "setoption",
                tokens_required: Requires::Args(4, 255),
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Leaf(Leaf::Fen), CommandConfig {
                token: "fen",
                tokens_required: Requires::Args(1, 255),
                parent_command: Command::Branch(Branch::Position),
                level: 2
            }),
            (Command::Leaf(Leaf::StartPos), CommandConfig {
                token: "startpos",
                tokens_required: Requires::Args(0, 255),
                parent_command: Command::Branch(Branch::Position),
                level: 2
            }),
            (Command::Leaf(Leaf::Perft), CommandConfig {
                token: "perft",
                tokens_required: Requires::Args(1, 1),
                parent_command: Command::Branch(Branch::Go),
                level: 2
            }),
            (Command::Leaf(Leaf::Display), CommandConfig {
                token: "display",
                tokens_required: Requires::None,
                parent_command: Command::Branch(Branch::Position),
                level: 2
            }),
            (Command::Leaf(Leaf::Move), CommandConfig {
                token: "move",
                tokens_required: Requires::Args(1, 255),
                parent_command: Command::Branch(Branch::Position),
                level: 2
            }),
            (Command::Leaf(Leaf::Undo), CommandConfig {
                token: "undo",
                tokens_required: Requires::Args(0, 1),
                parent_command: Command::Branch(Branch::Position),
                level: 2
            }),
            (Command::Leaf(Leaf::Uci), CommandConfig {
                token: "uci",
                tokens_required: Requires::None,
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Leaf(Leaf::UciNewGame), CommandConfig {
                token: "ucinewgame",
                tokens_required: Requires::None,
                parent_command: Command::Root,
                level: 1
            }),
            (Command::Leaf(Leaf::Depth), CommandConfig {
                token: "depth",
                tokens_required: Requires::Args(1, 1),
                parent_command: Command::Branch(Branch::Go),
                level: 2
            }),
            (Command::Leaf(Leaf::Help), CommandConfig {
                token: "help",
                tokens_required: Requires::Args(0, 255),
                parent_command: Command::Root,
                level: 1
            }),
        ])
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    Root,
    Branch(Branch),
    Leaf(Leaf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Branch {
    Position,
    Go,
}

/// Leaf commands are those that should be executed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Leaf {
    Quit,
    SetOption,
    Fen,
    StartPos,
    Perft,
    Display,
    Move,
    Undo,
    Uci,
    UciNewGame,
    Depth,
    Help,
}

impl Command {
    fn parse(token: &str, level: u8) -> Result<Self, ParseError> {
        to_lower!(token);
        for (key, config) in COMMAND_CONFIGS.iter() {
            // Level 0 will allow any level token to match
            if token == config.token && (level == config.level || level == 0) {
                return Ok(*key);
            }
        }
        return Err(ParseError::UnrecognisedTokens(token.to_string()));
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
    InvalidSubCommand(Command, Command),
    MissingTokens(Command),
    UnrecognisedTokens(String),
    InvalidFen(String),
    MissingArguments(Command, u8, u8),
    ExcessArguments(Command, u8, u8),
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
            }
            Self::UnrecognisedTokens(tokens) => format!("Unrecognised token(s): {tokens}"),
            Self::InvalidFen(msg) => format!("Invalid FEN string: {msg}"),
            Self::MissingArguments(token, min, n_tokens) => {
                let token = token.as_str();
                format!("Missing argument(s) for \"{token}\": {min} required, {n_tokens} provided")
            }
            Self::ExcessArguments(token, max, n_tokens) => {
                let token = token.as_str();
                format!("Too many arguments for \"{token}\": {max} allowed, {n_tokens} provided")
            }
        };
        log::error!("Could not parse command: {msg}");
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    ParseFenError(String),
    ParseAlgebraicError(String),
    InvalidMoveError(String, String),
    NullPromotionError(String, String),
}

impl ExecutionError {
    pub fn warn(&self) {
        let msg = match self {
            Self::ParseFenError(msg) => format!("Could not parse FEN: {msg}"),
            Self::ParseAlgebraicError(msg) => format!("Could not parse move: {msg}"),
            Self::InvalidMoveError(mv, fen) => {
                format!("Invalid move \"{mv}\" in the position {fen}")
            }
            Self::NullPromotionError(mv, fen) => {
                format!("Missing promotion specifier for \"{mv}\" in the position {fen}")
            }
        };
        log::error!("{msg}")
    }
}

pub struct CommandNode {
    pub cmd: Command,
    subcmds: Option<Vec<CommandNode>>,
    args: Option<Vec<String>>,
}

impl CommandNode {
    /// Tokenize and parse the input string into a Command struct
    pub fn parse(input: String) -> Result<Self, ParseError> {
        // Tokenize
        let input = input.trim();
        if input.len() == 0 {
            return Err(ParseError::NullInput);
        }
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let subcmds = Self::parse_subcommand_tokens(&Command::Root, &tokens, 1)?;
        // All commands are parsed as branches from the root command
        return Ok(Self {
            cmd: Command::Root,
            subcmds: Some(subcmds),
            args: None,
        });
    }

    fn parse_tokens(tokens: &Vec<&str>, level: u8) -> Result<Self, ParseError> {
        let cmd = Command::parse(tokens[0], level)?;
        let args = tokens[1..].to_vec();
        let n_tokens = args.len() as u8;
        // Check extra token requirements and build command struct accordingly
        match COMMAND_CONFIGS.get(&cmd).unwrap().tokens_required {
            Requires::SubCmd => {
                if n_tokens == 0 {
                    return Err(ParseError::MissingTokens(cmd));
                }
                let subcmd = Self::parse_subcommand_tokens(&cmd, &args, level + 1)?;
                Ok(Self {
                    cmd,
                    subcmds: Some(subcmd),
                    args: None,
                })
            }
            Requires::Args(min, max) => {
                if n_tokens < min {
                    return Err(ParseError::MissingArguments(cmd, min, n_tokens));
                }
                if n_tokens > max {
                    return Err(ParseError::ExcessArguments(cmd, max, n_tokens));
                }
                Self::check_arguments(&cmd, &args)?;
                let args = args.iter().map(|s| s.to_string()).collect();
                Ok(Self {
                    cmd,
                    subcmds: None,
                    args: Some(args),
                })
            }
            Requires::None => {
                if n_tokens > 0 {
                    return Err(ParseError::ExcessArguments(cmd, 0, n_tokens));
                }
                Ok(Self {
                    cmd,
                    subcmds: None,
                    args: None,
                })
            }
        }
    }

    fn parse_subcommand_tokens(
        cmd: &Command,
        tokens: &Vec<&str>,
        level: u8,
    ) -> Result<Vec<CommandNode>, ParseError> {
        // Parse the tokens into blocks of subcommands and their associated arguments
        let mut subcmds = Vec::new();
        let mut subcmd_stack = Vec::new();
        let mut arg_stack = Vec::new();
        for token in tokens.iter() {
            match Command::parse(token, level) {
                Ok(_) => {
                    if !subcmd_stack.is_empty() {
                        let mut subcmd_tokens = Vec::new();
                        subcmd_tokens.append(&mut subcmd_stack);
                        subcmd_tokens.append(&mut arg_stack);
                        subcmds.push(Self::parse_tokens(&subcmd_tokens, level)?);
                        Self::check_subcommand(&cmd, &subcmds.last().unwrap().cmd)?;
                    }
                    subcmd_stack.push(token);
                }
                Err(e) => {
                    if subcmd_stack.is_empty() {
                        // Return error if the first token is not a subcommand
                        return Err(e);
                    }
                    arg_stack.push(token)
                }
            }
        }
        // Flush stacks of the latest parsed command
        let mut subcmd_tokens = Vec::new();
        subcmd_tokens.append(&mut subcmd_stack);
        subcmd_tokens.append(&mut arg_stack);
        subcmds.push(Self::parse_tokens(&subcmd_tokens, level)?);
        Self::check_subcommand(&cmd, &subcmds.last().unwrap().cmd)?;
        Ok(subcmds)
    }

    /// Check that the subcommand provided is a valid option for the command
    /// * only should be invoked for commands requiring subcommands
    fn check_subcommand(cmd: &Command, subcmd: &Command) -> Result<(), ParseError> {
        if let Some(config) = COMMAND_CONFIGS.get(subcmd) {
            if *cmd != config.parent_command {
                return Err(ParseError::InvalidSubCommand(*cmd, *subcmd));
            }
            return Ok(());
        } else {
            log::error!("Command not in config dictionary");
            Err(ParseError::UnrecognisedTokens(subcmd.as_str().to_string()))
        }
    }

    /// Check that the arguments provided conform to the format expected
    /// * only should be invoked for commands requiring arguments
    fn check_arguments(cmd: &Command, args: &Vec<&str>) -> Result<(), ParseError> {
        if let Command::Leaf(cmd) = cmd {
            match cmd {
                Leaf::Fen => args_check::fen_tokens(args)?,
                Leaf::StartPos | Leaf::Move => args_check::move_tokens(args)?,
                Leaf::Undo => args_check::undo_token(args)?,
                Leaf::Perft => args_check::perft_token(args)?,
                Leaf::Display | Leaf::Uci | Leaf::UciNewGame | Leaf::Quit => (),
                Leaf::SetOption => log::warn!("setoption not implemented"), // TODO Implement SetOption
                Leaf::Depth => args_check::positive_numerical_token(args[0])?,
                Leaf::Help => args_check::help_tokens(args)?,
            }
        } else {
            log::warn!("Attempted argument parsing of non-leaf command")
        }
        Ok(())
    }

    pub fn execute(&self, state: &mut State) -> Result<(), ExecutionError> {
        // Only execute command if it's a leaf command i.e. no sub-command
        match &self.subcmds {
            Some(subcmds) => {
                for cmd in subcmds.iter() {
                    cmd.execute(state)?;
                }
            }
            None => self.execute_cmd(state)?,
        }
        Ok(())
    }

    fn execute_cmd(&self, state: &mut State) -> Result<(), ExecutionError> {
        if let Command::Leaf(cmd) = self.cmd {
            match cmd {
                Leaf::Perft => {
                    if let Some(token) = &self.args {
                        execute::perft(state, token[0].to_string())?
                    }
                }
                Leaf::Display => {
                    println!("{}", state.position.data.to_string());
                }
                Leaf::Fen => {
                    if let Some(args) = &self.args {
                        execute::fen(state, args)?
                    }
                }
                Leaf::StartPos => {
                    if let Some(args) = &self.args {
                        execute::startpos(state, args)?
                    }
                }
                Leaf::Move => {
                    if let Some(args) = &self.args {
                        execute::moves(state, args)?
                    }
                }
                Leaf::Undo => {
                    if let Some(args) = &self.args {
                        execute::undo(state, args)?
                    }
                }
                Leaf::Uci => execute::uci(state)?,
                Leaf::UciNewGame => execute::uci_new_game(state)?,
                Leaf::Quit => (),
                Leaf::SetOption => (), // TODO Implement
                Leaf::Depth => {
                    if let Some(args) = &self.args {
                        execute::depth_search(state, args[0].as_str())?
                    }
                },
                Leaf::Help => {
                    if let Some(args) = &self.args {
                        execute::help(args)?
                    }
                },
            }
            log::debug!("Command Executed {}", self.cmd.as_str())
        } else {
            log::warn!("Attempted execution of non-leaf command")
        }
        Ok(())
    }

    /// Print the parse tree
    /// * For debugging purposes
    pub fn print_parse_tree(&self, depth: usize) {
        println!(
            "{}Command=(\n{}{}",
            " ".repeat((depth - 1) * 4),
            " ".repeat((depth) * 4),
            self.cmd.as_str()
        );
        match &self.subcmds {
            Some(subcmds) => {
                for subcmd in subcmds.iter() {
                    subcmd.print_parse_tree(depth + 1);
                }
            }
            None => {
                if let Some(args) = &self.args {
                    println!("{}*args=({})", " ".repeat((depth) * 4), args.join(", "));
                }
            }
        }
        println!("{})", " ".repeat((depth - 1) * 4))
    }

    /// Traverse the parse tree and look for the presence of a "quit" token
    pub fn quit(&self) -> bool {
        if self.cmd == Command::Leaf(Leaf::Quit) {
            return true;
        }
        match &self.subcmds {
            Some(subcmds) => {
                for subcmd in subcmds.iter() {
                    if subcmd.cmd == Command::Leaf(Leaf::Quit) {
                        return true;
                    }
                }
                return false;
            }
            None => return false,
        }
    }
}

mod args_check {

    use super::*;

    pub fn fen_tokens(args: &Vec<&str>) -> Result<(), ParseError> {
        if args.len() < 6 {
            return Err(ParseError::InvalidFen(
                "Insufficient number of tokens".to_string(),
            ));
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
            'P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k', '/', '1', '2', '3', '4',
            '5', '6', '7', '8',
        ];
        if !token.chars().all(|c| VALID_CHARS.contains(&c)) {
            return Err(ParseError::InvalidFen(format!(
                "Invalid board token \"{token}\""
            )));
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
            return Err(ParseError::InvalidFen(format!(
                "Invalid board token \"{token}\""
            )));
        }
        Ok(())
    }

    fn wtm_token(token: &str) -> Result<(), ParseError> {
        if token == "w" || token == "b" {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!(
                "Invalid w.t.m. token \"{token}\""
            )))
        }
    }

    fn castle_token(token: &str) -> Result<(), ParseError> {
        const VALID_CHARS: [char; 4] = ['K', 'k', 'Q', 'q'];
        if (token
            .chars()
            .all(|c| VALID_CHARS.contains(&c) && token.len() <= 4))
            || token == "-"
        {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!(
                "Invalid castle token \"{token}\""
            )))
        }
    }

    fn ep_token(token: &str) -> Result<(), ParseError> {
        if ALGB_TOKEN.is_match(token) || token == "-" {
            Ok(())
        } else {
            Err(ParseError::InvalidFen(format!(
                "Invalid e.p. token \"{token}\""
            )))
        }
    }

    fn clock_token(token: &str) -> Result<(), ParseError> {
        if let Err(_) = token.parse::<u32>() {
            Err(ParseError::InvalidFen(format!(
                "Invalid clock token \"{token}\""
            )))
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
            return Err(ParseError::UnrecognisedTokens(invalid_tokens.join(" ")));
        }
        return Ok(());
    }

    pub fn perft_token(token: &Vec<&str>) -> Result<(), ParseError> {
        if !token[0].chars().all(|c| c.is_numeric()) && token[0] != "bench" {
            return Err(ParseError::UnrecognisedTokens(token[0].to_string()));
        }
        Ok(())
    }

    pub fn undo_token(token: &Vec<&str>) -> Result<(), ParseError> {
        if token.len() == 1 {
            if let Err(_) = token[0].parse::<u32>() {
                return Err(ParseError::UnrecognisedTokens(token[0].to_string()));
            }
        }
        Ok(())
    }

    pub fn positive_numerical_token(token: &str) -> Result<(), ParseError> {
        if let Err(_) = token.parse::<u32>() {
            Err(ParseError::UnrecognisedTokens(token.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn help_tokens(tokens: &Vec<&str>) -> Result<(), ParseError> {
        let mut valid_tokens = 0;
        let mut invalid_tokens = Vec::new();
        for token in tokens.iter() {
            match Command::parse(token, 0) {
                Ok(_) => valid_tokens += 1,
                Err(_) => invalid_tokens.push(*token)
            }
        }
        if valid_tokens == 0 && tokens.len() > 0 {
            Err(ParseError::UnrecognisedTokens(invalid_tokens.join(", ")))
        } else {
            Ok(())
        }
    }
}

mod execute {

    use super::*;

    pub fn perft(state: &mut State, arg: String) -> Result<(), ExecutionError> {
        let token = arg.parse::<i8>();
        match token {
            Ok(depth) => {
                search::perft::perft_divided(&state.position, depth, &state.config.perft_config);
            }
            Err(_) => search::perft::run_perft_bench(),
        };
        Ok(())
    }

    pub fn fen(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        let fen = args[0..6].join(" ");
        let moves = args[6..].to_vec();
        if fen != state.position.data.fen() {
            uci_new_game(state)?;
        }
        state.position_history.push(state.position.clone());
        state.position = position::Position::from_fen(fen)?;
        self::moves(state, &moves)
    }

    pub fn startpos(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        uci_new_game(state)?;
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
                        _ => {
                            return Err(ExecutionError::InvalidMoveError(
                                move_token.to_string(),
                                state.position.data.fen(),
                            ))
                        }
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
                        move_token.to_string(),
                        state.position.data.fen(),
                    ));
                }
                std::cmp::Ordering::Equal => {
                    // Execute the move
                    state.position_history.push(state.position.clone());
                    state.position = makemove::make_move(&state.position, &matches[0])
                }
                std::cmp::Ordering::Greater => {
                    // If there is more than 1 move, a promotion specifier was
                    // misiing
                    return Err(ExecutionError::NullPromotionError(
                        move_token.to_string(),
                        state.position.data.fen(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn undo(state: &mut State, args: &Vec<String>) -> Result<(), ExecutionError> {
        let n = if args.len() == 0 {
            1
        } else {
            args[0].parse::<u32>().unwrap()
        };
        let n = std::cmp::min(n, state.position_history.len() as u32);
        for _ in 0..n {
            if let Some(pos) = state.position_history.pop() {
                state.position = pos;
            }
        }
        Ok(())
    }

    pub fn uci(state: &mut State) -> Result<(), ExecutionError> {
        state.config.uci_mode = true;
        println!("id name LThink");
        println!("id author Leonard Lee");
        Ok(())
    }

    pub fn uci_new_game(state: &mut State) -> Result<(), ExecutionError> {
        state.position_history = Vec::new();
        state.transposition_table.clear();
        Ok(())
    }

    pub fn depth_search(state: &mut State, depth: &str) -> Result<(), ExecutionError> {
        let depth = depth.parse::<i8>().unwrap();
        search::do_search(
            &mut state.config,
            &state.position,
            depth,
            &mut state.transposition_table,
        );
        Ok(())
    }

    pub fn help(args: &Vec<String>) -> Result<(), ExecutionError> {
        if args.len() == 0 {
            // No additional arguments provided so list all available commands
            println!("\nAvailable Commands\n{}", "-".repeat(18));
            list_commands(&Command::Root, 0);
            println!("For more information on specific command usage, call \"help\", followed by the token");

            fn list_commands(cmd: &Command, level: u8) {
                for (subcmd, config) in COMMAND_CONFIGS.iter() {
                    if config.parent_command == *cmd && level == config.level {
                        println!(
                            "{}{}",
                            " ".repeat((level * 4) as usize),
                            config.token
                        );
                        if matches!(config.tokens_required, Requires::SubCmd) {
                            list_commands(subcmd, level+1)
                        };
                    }
                }
            }
        } else {
            let mut invalid_tokens = Vec::new();
            for arg in args.iter() {
                match Command::parse(arg.as_str(), 0) {
                    Ok(_) => (), // TODO Write help and display help
                    Err(_) => invalid_tokens.push(arg.as_str()),
                }
            }
            log::error!(
                "{} are not valid commands. For a list of valid commands, call \"help\"",
                invalid_tokens.join(", "),
            )
        }
        Ok(())
    }
}
