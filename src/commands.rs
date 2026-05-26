use crate::parse::{RESPError, RESPResult, RESPType};

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping,
    Echo(String),
    Set { key: String, value: String },
    Get(String),
}
impl Command {
    pub fn parse(request: Vec<RESPType>) -> RESPResult<Self> {
        // println!("{request:?}");
        let mut idx = 0;
        match request.get(idx) {
            Some(RESPType::BulkString(s)) => match &s.to_uppercase()[..] {
                "PING" => Ok(Self::Ping),
                "ECHO" => {
                    idx += 1;
                    match request.get(idx) {
                        Some(RESPType::BulkString(s)) => Ok(Self::Echo(s.to_string())),
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::MissingArg {
                            after: "ECHO".to_string(),
                        }),
                    }
                }
                "SET" => {
                    idx += 1;
                    let key = match request.get(idx) {
                        Some(RESPType::BulkString(s)) => Ok(s.to_string()),
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::MissingArg {
                            after: "SET".to_string(),
                        }),
                    }?;

                    idx += 1;
                    let value = match request.get(idx) {
                        Some(RESPType::BulkString(s)) => Ok(s.to_string()),
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::MissingArg { after: key.clone() }),
                    }?;

                    Ok(Self::Set { key, value })
                }
                "GET" => {
                    idx += 1;
                    match request.get(idx) {
                        Some(RESPType::BulkString(s)) => Ok(Self::Get(s.to_string())),
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::MissingArg {
                            after: "GET".to_string(),
                        }),
                    }
                }
                _ => Err(RESPError::InvalidCommand(s.to_string())),
            },
            Some(_) => unimplemented!(),
            None => return Err(RESPError::OutOfBounds(idx)),
        }
    }
}
