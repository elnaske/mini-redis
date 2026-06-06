use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::as_simple_string;
use crate::resp::errors::{RESPError, RESPResult};
use crate::resp::parse::RESPType;
use crate::storage::{KeyExpiry, Storage};

#[derive(Debug, PartialEq)]
pub struct Set {
    key: String,
    value: String,
    expire: Option<KeyExpiry>,
}
impl Set {
    pub fn new(key: String, value: String, expire: Option<KeyExpiry>) -> Self {
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
            Some(RESPType::BulkString(arg)) => match &arg.to_lowercase()[..] {
                "ex" => match args.next() {
                    Some(RESPType::BulkString(t)) => {
                        let t = t.parse::<u64>().map_err(|e| e.to_string()).unwrap();
                        Ok(Some(KeyExpiry::EX(t)))
                    }
                    Some(_) => Err(RESPError::CommandError),
                    None => Err(RESPError::MissingArgs),
                },
                "px" => match args.next() {
                    Some(RESPType::BulkString(t)) => {
                        let t = t.parse::<u64>().map_err(|e| e.to_string()).unwrap();
                        Ok(Some(KeyExpiry::PX(t)))
                    }
                    Some(_) => Err(RESPError::CommandError),
                    None => Err(RESPError::MissingArgs),
                },
                _ => Err(RESPError::CommandError),
            },
            Some(_) => Err(RESPError::CommandError),
            None => Ok(None),
        }?;

        Ok(Set::new(key.to_owned(), value.to_owned(), expire))
    }

    pub fn to_resp(self) -> RESPType {
        let mut arr = vec![
            RESPType::BulkString(String::from("SET")),
            RESPType::BulkString(self.key),
            RESPType::BulkString(self.value),
        ];
        if let Some(expire) = self.expire {
            arr.extend(expire.to_resp());
        }

        RESPType::Array(arr)
    }

    pub fn execute(self, storage: &Arc<Mutex<Storage>>) -> String {
        let mut storage = storage.lock().unwrap();

        if let Some(expire) = self.expire {
            let expiry_time = SystemTime::now() + expire.to_duration();
            storage.expiry.insert(self.key.clone(), expiry_time);
        }

        storage.db.insert(self.key, self.value);

        as_simple_string("OK")
    }
}
