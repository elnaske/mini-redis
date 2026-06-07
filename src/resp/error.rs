use std::fmt;

pub type RESPResult<T> = Result<T, RESPError>;

#[derive(Debug, PartialEq)]
pub enum RESPError {
    OutOfBounds(usize),
    InvalidPrefix(u8),
    InvalidCommand(String),
    InvalidSize(i32),
    ParseInt(String),
    MissingArgs,
    CommandError,
}
impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds(idx) => write!(f, "Index {} out of bounds", idx),
            Self::InvalidPrefix(c) => write!(f, "Invalid type prefix: {}", *c as char),
            Self::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            Self::InvalidSize(s) => write!(f, "Invalid length specifier: {}", s),
            Self::ParseInt(s) => write!(f, "Couldn't parse `{}` to an integer", s),
            Self::MissingArgs => write!(f, "Not enough arguments"),
            Self::CommandError => write!(f, "Command error"),
        }
    }
}
