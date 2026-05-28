use super::as_bulk_string;
use crate::parse::{RESPError, RESPResult, RESPType};
use crate::server::DB;

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
            Some(RESPType::BulkString(s)) => Ok(Get::new(s.to_string())),
            Some(_) => Err(RESPError::CommandError),
            None => Err(RESPError::MissingArgs),
        }
    }

    pub fn execute(&self, db: &DB) -> String {
        let db = db.lock().unwrap();
        match db.get(&self.key) {
            Some(value) => as_bulk_string(value),
            None => format!("+Error: Invalid Key `{}`\r\n", self.key),
        }
    }
}
