use crate::parse::{RESPError, RESPResult, RESPType};
use crate::server::DB;

mod ping;
pub use ping::Ping;

mod echo;
pub use echo::Echo;

mod set;
pub use set::Set;

mod get;
pub use get::Get;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
}
impl Command {
    pub fn parse(request: Vec<RESPType>) -> RESPResult<Self> {
        // println!("{request:?}");
        match request.first() {
            Some(RESPType::BulkString(s)) => match &s.to_uppercase()[..] {
                "PING" => Ok(Self::Ping(Ping::new())),
                "ECHO" => Ok(Self::Echo(Echo::parse(&request)?)),
                "SET" => Ok(Self::Set(Set::parse(&request)?)),
                "GET" => Ok(Self::Get(Get::parse(&request)?)),
                _ => Err(RESPError::InvalidCommand(s.to_string())),
            },
            Some(_) => unimplemented!(),
            None => Err(RESPError::MissingArgs),
        }
    }

    pub fn to_resp(self) -> String {
        match self {
            Self::Ping(ping) => ping.to_resp(),
            Self::Echo(echo) => echo.to_resp(),
            Self::Set(set) => set.to_resp(),
            Self::Get(get) => get.to_resp(),
        }
    }

    pub fn execute(self, db: &DB) -> String {
        match self {
            Self::Ping(ping) => ping.execute(),
            Self::Echo(echo) => echo.execute(),
            Self::Set(set) => set.execute(db),
            Self::Get(get) => get.execute(db),
        }
        // return response
    }
}

fn as_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

fn as_simple_string(s: &str) -> String {
    format!("+{}\r\n", s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn command_ping() {
        let cmd = Command::parse(vec![RESPType::BulkString("PING".to_string())]).unwrap();
        assert_eq!(cmd, Command::Ping(Ping::new()));
    }

    #[test]
    fn command_echo() {
        let cmd = Command::parse(vec![
            RESPType::BulkString("ECHO".to_string()),
            RESPType::BulkString("hello".to_string()),
        ])
        .unwrap();
        assert_eq!(cmd, Command::Echo(Echo::new("hello".to_string())));
    }

    #[test]
    fn command_set() {
        let cmd = Command::parse(vec![
            RESPType::BulkString("SET".to_string()),
            RESPType::BulkString("hello".to_string()),
            RESPType::BulkString("world".to_string()),
        ])
        .unwrap();
        assert_eq!(
            cmd,
            Command::Set(Set::new("hello".to_string(), "world".to_string()))
        );
    }

    #[test]
    fn command_get() {
        let cmd = Command::parse(vec![
            RESPType::BulkString("GET".to_string()),
            RESPType::BulkString("hello".to_string()),
        ])
        .unwrap();
        assert_eq!(cmd, Command::Get(Get::new("hello".to_string())));
    }

    #[test]
    fn command_invalid() {
        match Command::parse(vec![RESPType::BulkString("foo".to_string())]) {
            Err(RESPError::InvalidCommand(cmd)) => assert_eq!(cmd, "foo"),
            _ => panic!(),
        }
    }
}
