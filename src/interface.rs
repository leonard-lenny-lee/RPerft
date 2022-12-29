use super::*;
use state::State;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum CommandToken {
    // Level 0 Commands
    Position,
    Quit,
    Go,
    SetOption,
    // Level 1 Commands
    Fen,
    StartPos,
}

impl CommandToken {

    fn parse(token: &str, level: u8) -> Result<Self, ParseError> {
        match level {
            0 => match token {
                "position" => Ok(Self::Position),
                "quit" => Ok(Self::Quit),
                "go" => Ok(Self::Go),
                "setoption" => Ok(Self::SetOption),
                _ => Err(ParseError::InvalidCommand(token.to_string()))
            },
            1 => match token {
                "fen" => Ok(Self::Fen),
                "startpos" => Ok(Self::StartPos),
                _ => Err(ParseError::InvalidCommand(token.to_string()))
            },
            _ => Err(ParseError::UnrecognisedTokens(token.to_string()))
        }  
    }
}

enum Requires {
    SubCmd,
    Args,
    None
}

pub enum ParseError {
    NullInput,
    InvalidCommand(String),
    MissingTokens(CommandToken),
    UnrecognisedTokens(String),
}

impl ParseError {
    pub fn warn(&self) {
        match self {
            Self::NullInput => (),
            Self::InvalidCommand(msg) => (),
            Self::MissingTokens(token) => (),
            Self::UnrecognisedTokens(str) => ()
        };
    }
}

pub enum ExecutionError {
    
}

pub struct Command {
    cmd: CommandToken,
    subcmd: Option<Box<Command>>,
    args: Option<Vec<String>>,
}

impl Command {

    /// Tokenize and parse the input string into a Command struct
    pub fn parse(input: String) -> Result<Self, ParseError> {
        // Tokenize
        let binding = input.to_lowercase();
        let input = binding.trim();
        if input.len() == 0 {
            return Err(ParseError::NullInput)
        }
        let tokens: Vec<&str> = input.split_whitespace().collect();
        return Self::parse_tokens(&tokens, 0)
    }

    fn parse_tokens(tokens: &Vec<&str>, level: u8) -> Result<Self, ParseError> {
        let cmd = CommandToken::parse(tokens[0], level)?;
        let args = tokens[1..].to_vec();
        // Check extra token requirements
        match Self::check_extra_token_requirements(&cmd, &args)? {
            Requires::SubCmd => {
                let subcmd = Self::parse_tokens(tokens, level+1)?;
                Self::check_subcommand(&cmd, &subcmd.cmd)?;
                Ok(Self {cmd, subcmd: Some(Box::new(subcmd)), args: None})
            },
            Requires::Args => {
                Self::check_arguments(&cmd, &args)?;
                let args = args
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                Ok(Self {cmd, subcmd: None, args: Some(args)})
            },
            Requires::None => {
                Ok(Self {cmd, subcmd: None, args: None})
            }
        }
    }

    // Checks that for a specified command whether extra tokens are required,
    // allowed and the type of token required
    fn check_extra_token_requirements(
        cmd: &CommandToken, tokens: &Vec<&str>
    ) -> Result<Requires, ParseError> {
        use CommandToken::*;
        let (mandatory, allowed, requires) = match cmd {
            Position => (true, true, Requires::SubCmd),
            Quit => (false, false, Requires::None),
            Go => (true, true, Requires::SubCmd),
            SetOption => (true, true, Requires::Args),
            Fen => (true, true, Requires::Args),
            StartPos => (false, true, Requires::Args)
        };
        if mandatory && tokens.len() == 0 {
            return Err(ParseError::MissingTokens(*cmd))
        }
        if !allowed && tokens.len() >= 1 {
            return Err(ParseError::UnrecognisedTokens(tokens.join(" ")))
        }
        return Ok(requires)
    }

    /// Check that the subcommand provided is a valid option for the command
    fn check_subcommand(
        cmd: &CommandToken, subcmd: &CommandToken
    ) -> Result<(), ParseError> {
        Ok(())
    }

    /// Check that the arguments provided conform to the format expected for
    /// that command
    fn check_arguments(
        cmd: &CommandToken, args: &Vec<&str>
    ) -> Result<(), ParseError> {
        Ok(())
    }

    pub fn execute(&self, state: &mut State) -> Result<(), ExecutionError> {
        // Execute command if there is no subcommand; otherwise subcommand
        match &self.subcmd {
            Some(subcmd) => subcmd.execute(state),
            None => self.execute_cmd(state)
        }
    }

    fn execute_cmd(&self, state: &mut State) -> Result<(), ExecutionError> {
        Ok(())
    }

}
