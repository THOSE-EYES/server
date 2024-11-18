// A struct that stores info about user's active session
pub struct Session {
    pub user_id: i64,
    pub timestamp: i64,
}

impl Session {
    /// Create a new instance of Session
    pub fn new(user_id: i64, timestamp: i64) -> Self {
        Session { user_id, timestamp }
    }
}
