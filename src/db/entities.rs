use serde::Serialize;
use std::net::Ipv4Addr;
use std::time::Duration;

pub use i64 as ChatID;
pub use i64 as UserID;

/// A struture that mirrors the Users table in the database
#[derive(Serialize)]
pub struct User {
    pub id: UserID,
    pub name: String,
    pub surname: String,
    #[serde(skip)]
    pub password: String,
    #[serde(skip)]
    pub last_active: i64,
}

impl User {
    /// Create a new User instance
    pub fn new(
        id: UserID,
        name: String,
        surname: String,
        password: String,
        last_active: i64,
    ) -> User {
        User {
            id,
            name,
            surname,
            password,
            last_active,
        }
    }
}

/// A struture that mirrors the Chats table in the database
#[derive(Serialize)]
pub struct Chat {
    pub id: ChatID,
    pub title: String,
    pub description: String,
}

impl Chat {
    /// Create a new Chat instance
    pub fn new(id: ChatID, title: String, description: String) -> Chat {
        Chat {
            id,
            title,
            description,
        }
    }
}

/// A struture that mirrors the Invitations table in the database
#[derive(Serialize)]
pub struct Invitation {
    pub chat_id: ChatID,
    pub user_id: UserID,
}

impl Invitation {
    /// Create a new Invitations instance
    pub fn new(chat_id: ChatID, user_id: UserID) -> Invitation {
        Invitation { chat_id, user_id }
    }
}

/// A struture that mirrors the Devices table in the database
#[derive(Serialize)]
pub struct Device {
    user_id: UserID,
    pub ip: Ipv4Addr,
    pub name: String,
    pub is_active: bool,
}

impl Device {
    /// Create a new Devices instance
    pub fn new(user_id: UserID, ip: Ipv4Addr, name: String, is_active: bool) -> Device {
        Device {
            user_id,
            ip,
            name,
            is_active,
        }
    }
}

/// A struture that mirrors the Messages table in the database
#[derive(Serialize)]
pub struct Message {
    pub content: String,
    pub timestamp: Duration,
    pub chat_id: ChatID,
    pub user_id: UserID,
}

impl Message {
    /// Create a new Messages instance
    pub fn new(content: String, timestamp: Duration, chat_id: ChatID, user_id: UserID) -> Message {
        Message {
            content,
            timestamp,
            chat_id,
            user_id,
        }
    }
}
