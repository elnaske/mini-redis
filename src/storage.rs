use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub type DB = Arc<Mutex<Storage>>;

pub struct Storage {
    pub db: HashMap<String, String>,
    expiry: HashMap<String, SystemTime>,
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
