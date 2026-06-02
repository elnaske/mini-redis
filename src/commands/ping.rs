use crate::resp::parse::RESPType;

use super::as_simple_string;

#[derive(Debug, PartialEq, Default)]
pub struct Ping {}
impl Ping {
    pub fn new() -> Self {
        Ping {}
    }

    pub fn to_resp(self) -> RESPType {
        RESPType::Array(vec![RESPType::BulkString(String::from("PING"))])
    }

    pub fn execute(self) -> String {
        as_simple_string("PONG")
    }
}
