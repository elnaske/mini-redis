use super::as_bulk_string;
use crate::parse::{RESPError, RESPResult, RESPType};

#[derive(Debug, PartialEq)]
pub struct Echo {
    msg: String,
}
impl Echo {
    pub fn new(msg: String) -> Self {
        Echo { msg }
    }

    pub fn parse(request: &[RESPType]) -> RESPResult<Self> {
        match request.get(1) {
            Some(RESPType::BulkString(s)) => Ok(Echo::new(s.to_owned())),
            Some(_) => Err(RESPError::CommandError),
            None => Err(RESPError::MissingArgs),
        }
    }

    pub fn to_resp(self) -> String {
        format!(
            "*2\r\n$4\r\nECHO\r\n${}\r\n{}\r\n",
            self.msg.len(),
            self.msg
        )
    }

    pub fn execute(self) -> String {
        as_bulk_string(&self.msg)
    }
}
