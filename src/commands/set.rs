use super::as_simple_string;
use crate::parse::{RESPError, RESPResult, RESPType};
use crate::server::DB;

#[derive(Debug, PartialEq)]
pub struct Set {
    key: String,
    value: String,
}
impl Set {
    pub fn new(key: String, value: String) -> Self {
        Set { key, value }
    }

    pub fn parse(request: &[RESPType]) -> RESPResult<Self> {
        if request.len() < 3 {
            Err(RESPError::MissingArgs)
        } else if let RESPType::BulkString(k) = &request[1]
            && let RESPType::BulkString(v) = &request[2]
        {
            Ok(Set::new(k.to_owned(), v.to_owned()))
        } else {
            Err(RESPError::CommandError)
        }
    }

    pub fn to_resp(self) -> String {
        format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            self.key.len(),
            self.key,
            self.value.len(),
            self.value
        )
    }

    pub fn execute(self, db: &DB) -> String {
        let mut db = db.lock().unwrap();
        db.insert(self.key, self.value);
        as_simple_string("OK")
    }
}
