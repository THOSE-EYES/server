use std::collections::HashMap;
use std::fs::File;
use std::sync::Mutex;

use rand::random;

use crate::auth::Session;
use crate::db::{drivers::SQLite, Inserter, Retriever};
use crate::utils::unixepoch;

const DB_PATH: &'static str = "/tmp/test.db";

/// Contains all shared state of the server and implements core logic
pub struct App<T: Retriever + Inserter> {
    pub storage: Mutex<T>,
    pub sessions: Mutex<HashMap<i64, Session>>,
}

impl<T> App<T>
where
    T: Retriever + Inserter,
{
    /// Returns `user_id` for a valid session of that user
    pub fn session_validate_str(&self, session_id: &str) -> Option<i64> {
        let Ok(sid) = i64::from_str_radix(session_id, 10) else {
            return None;
        };
        let Ok(sessions) = self.sessions.lock() else {
            return None;
        };
        let Some(uid_ref) = sessions.get(&sid) else {
            return None;
        };
        Some(uid_ref.user_id)
    }
    /// Registers a new user to the database
    pub fn register(&self, name: &str, surname: &str, password: &str) -> Option<i64> {
        if let Ok(conn) = self.storage.lock() {
            let salt = format!("{:x}", random::<u64>());
            let mut saltpw = salt.clone();
            saltpw.push_str(password);

            let phash = blake3::hash(saltpw.as_bytes()).to_hex();
            if let Ok(id) = conn.create_user(name, surname, phash.as_str(), salt.as_str()) {
                conn.update_last_activity(id);
                return Some(id);
            }
        }
        None
    }

    pub fn login(&self, id: i64, password: &str) -> Option<i64> {
        if let Ok(conn) = self.storage.lock() {
            if let Ok(user) = conn.get_user(id) {
                let mut saltpw = user.salt.clone();
                saltpw.push_str(password);

                let phash = blake3::hash(saltpw.as_bytes()).to_hex();
                if user.password.eq(phash.as_str()) {
                    let session_id = random::<i32>() as i64;
                    let mut sessions = self.sessions.lock().unwrap();
                    sessions.insert(session_id, Session::new(id, unixepoch()));
                    return Some(session_id);
                }
            }
        }
        None
    }

    ///
    pub fn invite(&self, user_id: i64, chat_id: i64) -> Option<()> {
        if let Ok(conn) = self.storage.lock() {
            if let None = conn.add_user(chat_id, user_id) {
                return Some(());
            };
        }
        None
    }

    /// Creates a new chatroom in the database
    pub fn create_chat(&self, title: &str, description: &str) -> Option<i64> {
        if let Ok(conn) = self.storage.lock() {
            if let Ok(id) = conn.create_chat(title, description) {
                return Some(id);
            };
        }
        None
    }

    /// Stores a new message in the database
    pub fn message(&self, uid: i64, chat_id: i64, content: &str) -> Option<()> {
        if let Ok(conn) = self.storage.lock() {
            if let None = conn.store_message(chat_id, uid, content) {
                return Some(());
            };
        }
        None
    }

    pub fn set_activity(&self, sid: i64) -> Option<()> {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(v) = sessions.get_mut(&sid) {
                v.timestamp = unixepoch();
            };
        }
        if let Ok(conn) = self.storage.lock() {
            if let None = conn.update_last_activity(sid) {
                return Some(());
            };
        }
        None
    }

    pub fn is_active(&self, id: i64) -> Option<bool> {
        if let Ok(sessions) = self.sessions.lock() {
            match sessions.values().find(|e| (**e).user_id == id) {
                Some(_) => Some(true),
                None => Some(false),
            }
        } else {
            None
        }
    }

    pub fn logout(&self, sid: i64) -> Option<()> {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(&sid);
            Some(())
        } else {
            None
        }
    }

    pub fn reaper(&self) {
        let t = unixepoch();
        let mut sessions = self.sessions.lock().unwrap();
        let v: Vec<i64> = sessions
            .iter()
            .filter(|e| (e.1).timestamp + 90 < t)
            .map(|e| *e.0)
            .collect();
        for e in v {
            sessions.remove(&e);
        }
    }
}

impl App<SQLite> {
    /// Creates a new App based on an existing database.
    /// In case a database file is not found, it is created.
    pub fn new() -> Self {
        let _ = File::create_new(DB_PATH);
        App {
            storage: Mutex::new(SQLite::new(DB_PATH)),
            sessions: Mutex::new(HashMap::new()),
        }
    }
    /// Creates a new App along with a new database.
    /// In case a database file is found, it is overwritten.
    pub fn new_debug() -> Self {
        File::create(DB_PATH).unwrap(); // Truncate if exists
        App {
            storage: Mutex::new(SQLite::new(DB_PATH)),
            sessions: Mutex::new(HashMap::new()),
        }
    }
}
