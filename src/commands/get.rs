use super::as_bulk_string;
use crate::resp::errors::{RESPError, RESPResult};
use crate::resp::parse::RESPType;
use crate::storage::DB;

#[derive(Debug, PartialEq)]
pub struct Get {
    key: String,
}
impl Get {
    pub fn new(key: String) -> Self {
        Get { key }
    }

    pub fn parse(request: &[RESPType]) -> RESPResult<Self> {
        match request.get(1) {
            Some(RESPType::BulkString(s)) => Ok(Get::new(s.to_owned())),
            Some(_) => Err(RESPError::CommandError),
            None => Err(RESPError::MissingArgs),
        }
    }

    pub fn to_resp(self) -> RESPType {
        RESPType::Array(vec![
            RESPType::BulkString(String::from("GET")),
            RESPType::BulkString(self.key),
        ])
    }

    pub fn execute(self, storage: &DB) -> String {
        let storage = storage.lock().unwrap();
        match storage.db.get(&self.key) {
            Some(value) => as_bulk_string(value),
            None => format!("+Error: Invalid Key `{}`\r\n", self.key),
        }
    }
}
