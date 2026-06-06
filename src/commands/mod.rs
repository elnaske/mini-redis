use crate::resp::errors::{RESPError, RESPResult};
use crate::resp::parse::RESPType;
use crate::storage::DB;

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
        match request.first() {
            Some(RESPType::BulkString(s)) => match &s.to_uppercase()[..] {
                "PING" => Ok(Self::Ping(Ping::new())),
                "ECHO" => Ok(Self::Echo(Echo::parse(&request)?)),
                "SET" => Ok(Self::Set(Set::parse(&request)?)),
                "GET" => Ok(Self::Get(Get::parse(&request)?)),
                _ => Err(RESPError::InvalidCommand(s.to_owned())),
            },
            Some(_) => unimplemented!(),
            None => Err(RESPError::MissingArgs),
        }
    }

    pub fn to_resp(self) -> String {
        let request = match self {
            Self::Ping(ping) => ping.to_resp(),
            Self::Echo(echo) => echo.to_resp(),
            Self::Set(set) => set.to_resp(),
            Self::Get(get) => get.to_resp(),
        };
        request.to_string()
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
    use crate::storage::KeyExpiry;

    #[test]
    fn command_ping() {
        let cmd = Command::parse(vec![RESPType::BulkString(String::from("PING"))]).unwrap();
        assert_eq!(cmd, Command::Ping(Ping::new()));
    }

    #[test]
    fn command_echo() {
        let cmd = Command::parse(vec![
            RESPType::BulkString(String::from("ECHO")),
            RESPType::BulkString(String::from("hello")),
        ])
        .unwrap();
        assert_eq!(cmd, Command::Echo(Echo::new(String::from("hello"))));
    }

    #[test]
    fn command_set() {
        let cmd = Command::parse(vec![
            RESPType::BulkString(String::from("SET")),
            RESPType::BulkString(String::from("hello")),
            RESPType::BulkString(String::from("world")),
        ])
        .unwrap();
        assert_eq!(
            cmd,
            Command::Set(Set::new(String::from("hello"), String::from("world"), None))
        );
    }

    #[test]
    fn command_set_w_expiry() {
        let cmd = Command::parse(vec![
            RESPType::BulkString(String::from("SET")),
            RESPType::BulkString(String::from("hello")),
            RESPType::BulkString(String::from("world")),
            RESPType::BulkString(String::from("EX")),
            RESPType::BulkString(String::from("5")),
        ])
        .unwrap();
        assert_eq!(
            cmd,
            Command::Set(Set::new(
                String::from("hello"),
                String::from("world"),
                Some(KeyExpiry::EX(5))
            ))
        );

        let cmd = Command::parse(vec![
            RESPType::BulkString(String::from("SET")),
            RESPType::BulkString(String::from("hello")),
            RESPType::BulkString(String::from("world")),
            RESPType::BulkString(String::from("PX")),
            RESPType::BulkString(String::from("5")),
        ])
        .unwrap();
        assert_eq!(
            cmd,
            Command::Set(Set::new(
                String::from("hello"),
                String::from("world"),
                Some(KeyExpiry::PX(5))
            ))
        );
    }

    #[test]
    fn command_get() {
        let cmd = Command::parse(vec![
            RESPType::BulkString(String::from("GET")),
            RESPType::BulkString(String::from("hello")),
        ])
        .unwrap();
        assert_eq!(cmd, Command::Get(Get::new(String::from("hello"))));
    }

    #[test]
    fn command_invalid() {
        match Command::parse(vec![RESPType::BulkString(String::from("foo"))]) {
            Err(RESPError::InvalidCommand(cmd)) => assert_eq!(cmd, "foo"),
            _ => panic!(),
        }
    }
}
