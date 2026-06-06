use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::resp::RESPType;

pub type DB = Arc<Mutex<Storage>>;

pub struct Storage {
    pub db: HashMap<String, String>,
    pub expiry: HashMap<String, SystemTime>,
}
impl Storage {
    pub fn new() -> Self {
        Storage {
            db: HashMap::new(),
            expiry: HashMap::new(),
        }
    }

    pub fn expire_keys(&mut self) {
        let now = SystemTime::now();

        let expired = self
            .expiry
            .iter()
            .filter_map(|(k, v)| if *v < now { Some(k.clone()) } else { None })
            .collect::<Vec<String>>();

        for k in expired {
            self.db.remove(&k);
            self.expiry.remove(&k);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum KeyExpiry {
    EX(u64),
    PX(u64),
}
impl KeyExpiry {
    pub fn to_resp(self) -> [RESPType; 2] {
        match self {
            Self::EX(t) => [
                RESPType::BulkString(String::from("EX")),
                RESPType::BulkString(format!("{t}")),
            ],
            Self::PX(t) => [
                RESPType::BulkString(String::from("PX")),
                RESPType::BulkString(format!("{t}")),
            ],
        }
    }

    pub fn to_duration(self) -> Duration {
        match self {
            Self::EX(t) => Duration::from_secs(t),
            Self::PX(t) => Duration::from_millis(t),
        }
    }
}
