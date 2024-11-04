CREATE TABLE users(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    surname TEXT NOT NULL,
    password TEXT NOT NULL
    last_active INTEGER,
);

CREATE TABLE chats(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT
);

CREATE TABLE messages(
    content BLOB NOT NULL,
    timestamp INTEGER,
    chat_id INTEGER,
    user_id INTEGER
);

CREATE TABLE invitations(
    user_id INTEGER,
    chat_id INTEGER
);

CREATE TABLE devices(
    ip TEXT,
    name TEXT,
    user_id INTEGER,
    is_active INTEGER
); 

