use crate::db::{entities, DatabaseError, Retriever};

use sqlite::{Bindable, CursorWithOwnership};
use std::fs::read_to_string;
use std::net::Ipv4Addr;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

/// The file to use to re-create the database
const SCHEMA: &'static str = "db/schema.sql";

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
            read_to_string(SCHEMA)
                .unwrap()
                .lines()
                .for_each(|line| connection.execute(line).unwrap());
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
    /// match self.execute("SELECT id FROM users") {
    /// Ok(iter) => Ok(iter
    ///     .map(|row| row.unwrap().read::<UserID, _>("id"))
    ///     .collect()),
    /// Err(error) => Err(error),
    /// }
    /// ```
    fn execute(&self, query: &str) -> Result<CursorWithOwnership<'_>, DatabaseError> {
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
    /// match self.execute_parameterized(
    /// "SELECT * FROM invitations WHERE user_id = :id",
    /// [(":id", user_id)],
    /// ) {
    /// Ok(iter) => Ok(iter
    ///     .map(|row| row.unwrap().read::<ChatID, _>("chat_id"))
    ///     .collect()),
    /// Err(error) => Err(error),
    /// }
    /// ```
    fn execute_parameterized<T, U>(
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
        match self.execute("SELECT * FROM users") {
            Ok(iter) => Ok(iter
                .map(|result| {
                    let row = result.unwrap();

                    entities::User::new(
                        row.read::<entities::UserID, _>("id"),
                        String::from(row.read::<&str, _>("name")),
                        String::from(row.read::<&str, _>("surname")),
                        String::from(row.read::<&str, _>("password")),
                    )
                })
                .collect()),
            Err(error) => Err(error),
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
        match self.execute_parameterized(
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
        match self.execute_parameterized(
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
        match self.execute_parameterized(
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
