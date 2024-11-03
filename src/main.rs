use axum::{
    extract::{Json, Query, State},
    routing::{get, post},
    Router,
};
use rand::random;
use std::collections::HashMap;
use std::string::String;
use std::sync::{Arc, Mutex};

mod db;
use db::{drivers::SQLite, Retriever};
use std::fs::File;

const DB_PATH: &'static str = "/tmp/test.db";

pub struct App<T: Retriever> {
    storage: Mutex<T>,
    sessions: Mutex<HashMap<i32, i32>>,
}
impl App<SQLite> {
    pub fn new() -> Self {
        let r = File::create_new(DB_PATH);
        App {
            storage: Mutex::new(SQLite::new(DB_PATH)),
            sessions: Mutex::new(HashMap::new()),
        }
    }
    pub fn new_debug() -> Self {
        File::create(DB_PATH).unwrap(); // Truncate if exists
        App {
            storage: Mutex::new(SQLite::new(DB_PATH)),
            sessions: Mutex::new(HashMap::new()),
        }
    }
    //    pub fn register(&self, name: &str, password: &str) -> Option<()> {
    //        if let Ok(conn) = self.db.lock() {
    //            // Check if {name} exists
    //            let mut statement = conn
    //                .prepare(format!("SELECT * FROM users WHERE name = {:?};", name))
    //                .unwrap();
    //            if let Ok(sqlite::State::Row) = statement.next() {
    //                return None;
    //            }
    //            // Modify db
    //            conn.execute(format!(
    //                "INSERT INTO users VALUES(NULL, {:?}, '{:?}');",
    //                name, password,
    //            ))
    //            .unwrap();
    //        }
    //        Some(())
    //    }

    //pub fn login(&self, name: &str, password: &str) -> Option<i64> {
    //if let Ok(conn) = self.db.lock() {
    //let mut statement = conn
    //.prepare(format!(
    //"SELECT id, surname FROM users WHERE name = '{:?}';",
    //name
    //))
    //.unwrap();
    //if let Ok(sqlite::State::Row) = statement.next() {
    //if statement.read::<&str, _>("password").unwrap().eq(password) {
    //let session_id = random::<i32>();
    //let user_id = statement.read::<i64, _>("password").unwrap() as i32;
    //let mut sessions = self.sessions.lock().unwrap();
    //sessions.insert(session_id, user_id);
    //return Some(session_id);
    //}
    //}
    //}
    //None
    //}
}

async fn g_users<T: Retriever>(State(state): State<Arc<App<T>>>) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    if let Ok(list) = db.get_users() {
        sb.push_str(r#"{"users":["#);
        for e in &list {
            sb.push_str(
                format!(
                    r#"{{"user_id":{},"name":"{}","surname":"{}"}},"#,
                    e.id, e.name, e.surname
                )
                .as_str(),
            );
        }
        // Remove trailing ','
        if !list.is_empty() {
            sb.pop();
        }
        sb.push_str(r#"]}"#);
    }
    sb
}

async fn g_chats<T: Retriever>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let uid = i64::from_str_radix(params.get("user_id").expect("No param").as_str(), 10).unwrap();
    if let Ok(list) = db.get_chats(uid) {
        sb.push_str(r#"{"chats":["#);
        for e in &list {
            sb.push_str(
                format!(
                    r#"{{"chat_id":{},"title":"{}","description":"{}"}}"#,
                    e.id, e.title, e.description
                )
                .as_str(),
            );
        }
        if !list.is_empty() {
            sb.pop();
        } // Remove trailing ','
        sb.push_str(r#"]}"#);
    }
    sb
}

async fn g_messages<T: Retriever>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let uid = i64::from_str_radix(params.get("user_id").expect("No param").as_str(), 10).unwrap();
    if let Ok(list) = db.get_messages(uid) {
        sb.push_str(r#"{"messages":["#);
        for e in &list {
            sb.push_str(
                format!(r#"{{"chat_id":{},"content":"{}"}}"#, e.chat_id, e.content).as_str(),
            );
        }
        if !list.is_empty() {
            sb.pop();
        } // Remove trailing ','
        sb.push_str(r#"]}"#);
    }
    sb
}

async fn g_devices<T: Retriever>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let uid = i64::from_str_radix(params.get("user_id").expect("No param").as_str(), 10).unwrap();
    if let Ok(list) = db.get_devices(uid) {
        sb.push_str(r#"{"devices":["#);
        for e in &list {
            sb.push_str(format!(r#"{{"name":"{}","is_active":{}}}"#, e.name, e.is_active).as_str());
        }
        if !list.is_empty() {
            sb.pop();
        } // Remove trailing ','
        sb.push_str(r#"]}"#);
    }
    sb
}

//async fn p_register(
//    State(state): State<Arc<App<T>>>,
//    Json(payload): Json<serde_json::Value>,
//) -> &'static str {
//    if let (Some(name), Some(password)) = (payload["name"].as_str(), payload["password"].as_str()) {
//        if let Some(()) = state.register(name, password) {
//            return r#"{"status":"Ok"}"#;
//        }
//    }
//    r#"{"status":"Err"}"#
//}

//async fn p_login<T: Retriever>(
//State(state): State<Arc<App<T>>>,
//Json(payload): Json<serde_json::Value>,
//) -> String {
//if let (Some(name), Some(password)) = (payload["name"].as_str(), payload["password"].as_str()) {
//if let Some(session_id) = state.login(name, password) {
//return format!(r#"{{"status":"Ok", "session_id": {}}}"#, session_id);
//}
//}
//String::from(r#"{"status":"Err"}"#)
//}

#[tokio::main]
async fn main() {
    let app = Arc::new(App::new_debug());
    let router = Router::new()
        .route("/users", get(g_users::<SQLite>))
        .route("/chats", get(g_chats::<SQLite>))
        .route("/messages", get(g_messages::<SQLite>))
        .route("/devices", get(g_devices::<SQLite>))
        //.route("/users", post(p_register))
        //.route("/login", post(p_login))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
