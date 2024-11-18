use crate::db::{entities, DatabaseError, Inserter, Retriever};

use sqlite::{Bindable, CursorWithOwnership};
use std::net::Ipv4Addr;
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

/// The file to use to re-create the database
const SCHEMA: &'static str = include_str!("../../../db/schema.sql");

/// A concrete driver wrapper that handles SQLite databases
pub struct SQLite {
    // A handler that is used to use the connection to the SQLite database
    handler: sqlite::Connection,
}

impl SQLite {
    /// Create a new instance of SQLite struct
    pub fn new(path: &str) -> SQLite {
        // Check if the database exists
        let flag = !Path::new(path).exists();
        let connection = sqlite::open(path).unwrap();

        // Re-create the database if necessary
        if !flag {
            connection.execute(SCHEMA).unwrap();
        }

        SQLite {
            handler: connection,
        }
    }

    /// Execute a query without parameters and return the results
    ///
    /// This method prepares a statement based on the query it receives from
    /// the user, runs it and returns the cursor, which is needed to map and
    /// collect the values.
    ///
    /// # Examples
    /// ```
    /// match self.prepare("SELECT id FROM users") {
    /// Ok(iter) => Ok(iter
    ///     .map(|row| row.unwrap().read::<UserID, _>("id"))
    ///     .collect()),
    /// Err(error) => Err(error),
    /// }
    /// ```
    fn prepare(&self, query: &str) -> Result<CursorWithOwnership<'_>, DatabaseError> {
        match self.handler.prepare(query) {
            Ok(statement) => Ok(statement.into_iter()),
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Duplicate function for external usage TEMPORARY
    pub fn execute(&self, query: &str) -> Result<CursorWithOwnership<'_>, DatabaseError> {
        match self.handler.prepare(query) {
            Ok(statement) => Ok(statement.into_iter()),
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Execute a parameterized query and return the results
    ///
    /// This method prepares a statement based on the query it receives from
    /// the user, runs it with the given parameters and returns the cursor,
    /// which is needed to map and collect the values.
    ///
    /// # Examples
    /// ```
    /// match self.prepare_parameterized(
    /// "SELECT * FROM invitations WHERE user_id = :id",
    /// [(":id", user_id)],
    /// ) {
    /// Ok(iter) => Ok(iter
    ///     .map(|row| row.unwrap().read::<ChatID, _>("chat_id"))
    ///     .collect()),
    /// Err(error) => Err(error),
    /// }
    /// ```
    fn prepare_parameterized<T, U>(
        &self,
        query: &str,
        bind_value: T,
    ) -> Result<CursorWithOwnership<'_>, DatabaseError>
    where
        T: IntoIterator<Item = U>,
        U: Bindable,
    {
        match self.handler.prepare(query) {
            Ok(statement) => match statement.into_iter().bind_iter(bind_value) {
                Ok(iter) => Ok(iter),
                Err(error) => Err(DatabaseError::new(error.message.unwrap())),
            },
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Execute a parameterized query without returning results
    ///
    /// This method prepares a statement based on the query it receives from
    /// the user, runs it with the given parameters and returns errors if any.
    ///
    /// # Examples
    /// ```
    /// let query = "INSERT INTO messages VALUES(:content, :timestamp, :chat_id, :user_id)";
    /// let timestamp = SystemTime::now()
    ///     .duration_since(SystemTime::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_millis();
    ///
    /// self.execute_parameterized(
    ///     query,
    ///     [
    ///         (":content", content.as_str()),
    ///         (":timestamp", &timestamp.to_string()),
    ///         (":chat_id", &chat_id.to_string()),
    ///         (":user_id", &user_id.to_string()),
    ///     ],
    /// )
    /// ```
    fn execute_parameterized<T, U>(&self, query: &str, bind_value: T) -> Option<DatabaseError>
    where
        T: IntoIterator<Item = U>,
        U: Bindable,
    {
        match self.handler.prepare(query) {
            Ok(mut statement) => match statement.bind_iter(bind_value) {
                Ok(_) => match statement.next() {
                    Ok(_) => None,
                    Err(error) => Some(DatabaseError::new(error.message.unwrap())),
                },
                Err(error) => Some(DatabaseError::new(error.message.unwrap())),
            },
            Err(error) => Some(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Get filled Chat structure instance for the chat with the given ID.
    ///
    /// The method uses the provided ID to get all the information about the
    /// chat from the database.
    ///
    /// # Examples
    /// ```
    /// let driver = SQLite::new("database.db");
    /// let chat = driver.get_chat(id).unwrap();
    /// ```
    fn get_chat(&self, id: entities::ChatID) -> Result<entities::Chat, DatabaseError> {
        let query = "SELECT * FROM chats WHERE id = :id";

        match self.handler.prepare(query) {
            Ok(mut statement) => match statement.bind((":id", id)) {
                Ok(_) => {
                    if let Err(error) = statement.next() {
                        Err(DatabaseError::new(error.message.unwrap()))
                    } else {
                        Ok(entities::Chat::new(
                            statement.read::<i64, _>("id").unwrap(),
                            statement.read::<String, _>("title").unwrap(),
                            statement.read::<String, _>("description").unwrap(),
                        ))
                    }
                }
                Err(error) => Err(DatabaseError::new(error.message.unwrap())),
            },
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }
}

impl Retriever for SQLite {
    /// Get a list of users
    ///
    /// The method reads the list of users, which are avaliable in the
    /// database.
    ///
    /// # Examples
    /// ```
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_users().unwrap() {
    ///     println!("User with the ID found: {}", value.id);
    /// }
    /// ```
    fn get_users(&self) -> Result<Vec<entities::User>, DatabaseError> {
        match self.prepare("SELECT * FROM users") {
            Ok(iter) => Ok(iter
                .map(|result| {
                    let row = result.unwrap();

                    entities::User::new(
                        row.read::<entities::UserID, _>("id"),
                        String::from(row.read::<&str, _>("name")),
                        String::from(row.read::<&str, _>("surname")),
                        String::from(row.read::<&str, _>("password")),
                        row.read::<i64, _>("last_active"),
                    )
                })
                .collect()),
            Err(error) => Err(error),
        }
    }

    /// Get the user info
    ///
    /// The method reads the list of users, which are avaliable in the
    /// database and returns the one with the given ID.
    ///
    /// # Examples
    /// ```
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_user(0).unwrap() {
    ///     println!("User with the name found: {}", value.name);
    /// }
    /// ```
    fn get_user(&self, user_id: entities::UserID) -> Result<entities::User, DatabaseError> {
        let query = "SELECT * FROM users WHERE id = :id";
        match self.handler.prepare(query) {
            Ok(mut statement) => match statement.bind((":id", user_id)) {
                Ok(_) => {
                    if let Err(error) = statement.next() {
                        Err(DatabaseError::new(error.message.unwrap()))
                    } else {
                        Ok(entities::User::new(
                            statement.read::<i64, _>("id").unwrap(),
                            statement.read::<String, _>("name").unwrap(),
                            statement.read::<String, _>("surname").unwrap(),
                            statement.read::<String, _>("password").unwrap(),
                            statement.read::<i64, _>("last_active").unwrap(),
                        ))
                    }
                }
                Err(error) => Err(DatabaseError::new(error.message.unwrap())),
            },
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Get a list of chats, available for the user
    ///
    /// The method reads the list of all the chats, which are avaliable for the
    /// specified user.
    ///
    /// # Examples
    /// ```
    /// let user_id = 0;
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_chats(user_id).unwrap() {
    ///     println!("User {} has access to the chat with ID: {}", user_id, value.id);
    /// }
    /// ```
    fn get_chats(&self, user_id: entities::UserID) -> Result<Vec<entities::Chat>, DatabaseError> {
        match self.prepare_parameterized(
            "SELECT * FROM invitations WHERE user_id = :id",
            [(":id", user_id)],
        ) {
            Ok(iter) => Ok(iter
                .map(|result| {
                    let row = result.unwrap();
                    let id = row.read::<entities::ChatID, _>("chat_id");

                    self.get_chat(id).unwrap()
                })
                .collect()),
            Err(error) => Err(error),
        }
    }

    /// Get a list of messages, available for the user
    ///
    /// The method reads the list of all the chats, which are avaliable for the
    /// specified user.
    ///
    /// # Examples
    /// ```
    /// let user_id = 0;
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_chats(user_id).unwrap() {
    ///     println!("User {} has access to the chat with ID: {}", user_id, value);
    /// }
    /// ```
    fn get_messages(
        &self,
        chat_id: entities::ChatID,
    ) -> Result<Vec<entities::Message>, DatabaseError> {
        match self.prepare_parameterized(
            "SELECT * FROM messages WHERE chat_id = :id",
            [(":id", chat_id)],
        ) {
            Ok(iter) => Ok(iter
                .map(|result| {
                    let row = result.unwrap();

                    entities::Message::new(
                        String::from(row.read::<&str, _>("content")),
                        Duration::from_millis(row.read::<i64, _>("timestamp") as u64),
                        row.read::<entities::ChatID, _>("chat_id"),
                        row.read::<entities::UserID, _>("user_id"),
                    )
                })
                .collect()),
            Err(error) => Err(error),
        }
    }

    /// Get a list of devices, associated with the user
    ///
    /// The method reads the list of all the devices, that were logged in with
    /// the given user
    ///
    /// # Examples
    /// ```
    /// let user_id = 0;
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_devices(user_id).unwrap() {
    ///     println!("User {} has logged in from the following device: {}", user_id, value.name);
    /// }
    /// ```
    fn get_devices(
        &self,
        user_id: entities::UserID,
    ) -> Result<Vec<entities::Device>, DatabaseError> {
        match self.prepare_parameterized(
            "SELECT * FROM devices WHERE user_id = :id",
            [(":id", user_id)],
        ) {
            Ok(iter) => Ok(iter
                .map(|result| {
                    let row = result.unwrap();

                    entities::Device::new(
                        row.read::<entities::UserID, _>("user_id"),
                        Ipv4Addr::from_str(row.read::<&str, _>("ip")).unwrap(),
                        String::from(row.read::<&str, _>("name")),
                        row.read::<i64, _>("is_active") != 0,
                    )
                })
                .collect()),
            Err(error) => Err(error),
        }
    }
}

impl Inserter for SQLite {
    /// Store the message in the database
    ///
    /// This method stores the message with the given content in the chat
    /// that the user sent.
    ///
    /// # Examples
    /// ```
    /// let driver = drivers::SQLite::new("database.db");
    /// if let Some(error) = driver.store_message(0, 0, "B".to_string()) {
    ///     println!("{}", error.message);
    /// } else {
    ///     println!("No errors");
    /// }
    /// ```
    fn store_message(
        &self,
        chat_id: entities::ChatID,
        user_id: entities::UserID,
        content: &str,
    ) -> Option<DatabaseError> {
        let query = "INSERT INTO messages VALUES(:content, :timestamp, :chat_id, :user_id)";
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.execute_parameterized(
            query,
            [
                (":content", content),
                (":timestamp", &timestamp.to_string()),
                (":chat_id", &chat_id.to_string()),
                (":user_id", &user_id.to_string()),
            ],
        )
    }

    /// Create a new user
    ///
    /// This method updates the database with the user, defined by the
    /// parameters supplied to the method. The ID of the user is returned.
    ///
    /// # Examples
    /// ```
    /// let driver = drivers::SQLite::new("database.db");
    /// println!(
    ///     "User with the ID {} created.",
    ///     driver
    ///         .create_user(
    ///             "name".to_string(),
    ///             "surname".to_string(),
    ///             "password".to_string()
    ///         )
    ///         .unwrap()
    /// );
    /// ```
    fn create_user(
        &self,
        name: &str,
        surname: &str,
        password: &str,
    ) -> Result<entities::UserID, DatabaseError> {
        let query =
        "INSERT INTO users(name, surname, password, last_active) VALUES(:name,:surname,:password,unixepoch()) RETURNING id";

        match self.handler.prepare(query) {
            Ok(mut statement) => match statement.bind_iter([
                (":name", name),
                (":surname", surname),
                (":password", password),
            ]) {
                Ok(_) => {
                    if let Err(error) = statement.next() {
                        Err(DatabaseError::new(error.message.unwrap()))
                    } else {
                        Ok(statement.read::<i64, _>(0).unwrap())
                    }
                }
                Err(error) => Err(DatabaseError::new(error.message.unwrap())),
            },
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Create a new chat
    ///
    /// This method updates the database with the chat, defined by the
    /// parameters supplied to the method. The ID of the chat is returned.
    ///
    /// # Examples
    /// ```
    /// let driver = drivers::SQLite::new("database.db");
    /// println!(
    ///     "Chat with the ID {} created.",
    ///     driver
    ///         .create_chat(
    ///             "title".to_string(),
    ///             "description".to_string(),
    ///         )
    ///         .unwrap()
    /// );
    /// ```
    fn create_chat(
        &self,
        title: &str,
        description: &str,
    ) -> Result<entities::ChatID, DatabaseError> {
        let query =
            "INSERT INTO chats(title, description) VALUES(:title,:description) RETURNING id";

        match self.handler.prepare(query) {
            Ok(mut statement) => {
                match statement.bind_iter([(":title", title), (":description", description)]) {
                    Ok(_) => {
                        if let Err(error) = statement.next() {
                            Err(DatabaseError::new(error.message.unwrap()))
                        } else {
                            Ok(statement.read::<i64, _>(0).unwrap())
                        }
                    }
                    Err(error) => Err(DatabaseError::new(error.message.unwrap())),
                }
            }
            Err(error) => Err(DatabaseError::new(error.message.unwrap())),
        }
    }

    /// Add a user to the chat
    ///
    /// This method adds the user with the given ID to the chat with the given
    /// ID by writing new data to the database.
    ///
    /// # Examples
    /// ```
    /// let driver = drivers::SQLite::new("database.db");
    /// if let Some(error) = driver.add_user(0, 0) {
    ///     println!("{}", error.message);
    /// } else {
    ///     println!("No errors");
    /// }
    /// ```
    fn add_user(
        &self,
        chat_id: entities::ChatID,
        user_id: entities::UserID,
    ) -> Option<DatabaseError> {
        let query = "INSERT INTO invitations VALUES(:chat_id, :user_id)";

        self.execute_parameterized(
            query,
            [
                (":chat_id", chat_id.to_string().as_str()),
                (":user_id", user_id.to_string().as_str()),
            ],
        )
    }

    /// Update the last activity timestamp of the user
    ///
    /// This method gets the current time as a UNIX timestamp and updates the
    /// 'last_active' field of the users table for the given user_id
    ///
    /// # Examples
    /// ```
    /// let driver = drivers::SQLite::new("database.db");
    /// if let Some(error) = driver.update_last_activity(0) {
    ///     println!("{}", error.message);
    /// } else {
    ///     println!("No errors");
    /// }
    /// ```    
    fn update_last_activity(&self, user_id: entities::UserID) -> Option<DatabaseError> {
        let query = "UPDATE users SET last_active = unixepoch() WHERE user_id = :id";
        self.execute_parameterized(query, [(":user_id", user_id.to_string().as_str())])
    }
}
