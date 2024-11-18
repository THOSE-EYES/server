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
    fn get_user(&self, user_id: entities::UserID) -> Result<entities::User, DatabaseError>;

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

/// A trait for all the structs that update databases
pub trait Inserter {
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
    ) -> Option<DatabaseError>;

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
        salt: &str,
    ) -> Result<entities::UserID, DatabaseError>;

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
    ) -> Result<entities::ChatID, DatabaseError>;

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
    ) -> Option<DatabaseError>;

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
    fn update_last_activity(&self, user_id: entities::UserID) -> Option<DatabaseError>;
}
