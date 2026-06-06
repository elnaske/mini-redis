use std::time::Duration;

use super::as_simple_string;
use crate::resp::errors::{RESPError, RESPResult};
use crate::resp::parse::RESPType;
use crate::storage::DB;

#[derive(Debug, PartialEq)]
pub struct Set {
    key: String,
    value: String,
    expire: Option<Duration>,
}
impl Set {
    pub fn new(key: String, value: String, expire: Option<Duration>) -> Self {
        Set { key, value, expire }
    }

    pub fn parse(request: &[RESPType]) -> RESPResult<Self> {
        let mut args = request.iter().skip(1);

        let Some(RESPType::BulkString(key)) = args.next() else {
            return Err(RESPError::MissingArgs);
        };
        let Some(RESPType::BulkString(value)) = args.next() else {
            return Err(RESPError::MissingArgs);
        };

        let expire = match args.next() {
            Some(RESPType::BulkString(arg)) => {
                match &arg.to_lowercase()[..] {
                    // expiration time
                    "ex" | "px" => match args.next() {
                        Some(RESPType::BulkString(t)) => {
                            let t = {
                                let t = t.parse::<u64>().map_err(|e| e.to_string()).unwrap();
                                if arg == "ex" {
                                    Duration::from_secs(t)
                                } else {
                                    Duration::from_millis(t)
                                }
                            };
                            Ok(Some(t))
                        }
                        Some(_) => Err(RESPError::CommandError),
                        None => Err(RESPError::MissingArgs),
                    },
                    _ => Err(RESPError::CommandError),
                }
            }
            Some(_) => Err(RESPError::CommandError),
            None => Ok(None),
        }?;

        Ok(Set::new(key.to_owned(), value.to_owned(), expire))
    }

    pub fn to_resp(self) -> RESPType {
        RESPType::Array(vec![
            RESPType::BulkString(String::from("SET")),
            RESPType::BulkString(self.key),
            RESPType::BulkString(self.value),
        ])
    }

    pub fn execute(self, storage: &DB) -> String {
        let mut storage = storage.lock().unwrap();
        storage.db.insert(self.key, self.value);
        as_simple_string("OK")
    }
}
