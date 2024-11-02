pub mod drivers;
pub mod entities;

/// A structure that is used to unify errors got from the driver implementation
#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
}

impl DatabaseError {
    /// Create a new instance of the database error
    fn new(message: String) -> DatabaseError {
        DatabaseError { message }
    }
}

/// A public trait, that is used to implement access to the database for the
/// GET requests
pub trait Retriever {
    /// Get a list of users
    ///
    /// The method reads the list of users, which are avaliable in the
    /// database.
    ///
    /// # Examples
    /// ```
    /// let driver = SQLite::new("data.db");
    /// for value in driver.get_users().unwrap() {
    ///     println!("User with the ID found: {}", value);
    /// }
    /// ```
    fn get_users(&self) -> Result<Vec<entities::User>, DatabaseError>;

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
    ///     println!("User {} has access to the chat with ID: {}", user_id, value);
    /// }
    /// ```
    fn get_chats(&self, user_id: entities::UserID) -> Result<Vec<entities::Chat>, DatabaseError>;

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
    ) -> Result<Vec<entities::Message>, DatabaseError>;

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
    ) -> Result<Vec<entities::Device>, DatabaseError>;
}
