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
use db::{drivers::SQLite, Inserter, Retriever};
use std::fs::File;

const DB_PATH: &'static str = "/tmp/test.db";

pub struct App<T: Retriever + Inserter> {
    storage: Mutex<T>,
    sessions: Mutex<HashMap<i64, i64>>,
}
impl<T> App<T>
where
    T: Retriever + Inserter,
{
    fn session_validate_str(&self, session_id: &str) -> Option<i64> {
        let Ok(sid) = i64::from_str_radix(session_id, 10) else {
            return None;
        };
        let Ok(sessions) = self.sessions.lock() else {
            return None;
        };
        let Some(uid_ref) = sessions.get(&sid) else {
            return None;
        };
        Some(uid_ref.clone())
    }
    fn register(&self, name: &str, surname: &str, password: &str) -> Option<()> {
        if let Ok(conn) = self.storage.lock() {
            if let Ok(_) = conn.create_user(name, surname, password) {
                return Some(());
            }
        }
        None
    }
    fn invite(&self, user_id: i64, chat_id: i64) -> Option<()> {
        if let Ok(conn) = self.storage.lock() {
            if let None = conn.add_user(chat_id, user_id) {
                return Some(());
            };
        }
        None
    }

    fn make_chat(&self, title: &str, description: &str) -> Option<i64> {
        if let Ok(conn) = self.storage.lock() {
            if let Ok(id) = conn.create_chat(title, description) {
                return Some(id);
            };
        }
        None
    }

    fn message(&self, uid: i64, chat_id: i64, content: &str) -> Option<()> {
        if let Ok(conn) = self.storage.lock() {
            if let None = conn.store_message(chat_id, uid, content) {
                return Some(());
            };
        }
        None
    }

    fn login(&self, id: i64, password: &str) -> Option<i64> {
        if let Ok(conn) = self.storage.lock() {
            if let Ok(user) = conn.get_user(id) {
                if user.password.eq(password) {
                    let session_id = random::<i64>();
                    let mut sessions = self.sessions.lock().unwrap();
                    sessions.insert(session_id, id);
                    return Some(session_id);
                }
            }
        }
        None
    }
}
impl App<SQLite> {
    pub fn new() -> Self {
        let _ = File::create_new(DB_PATH);
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
}

async fn g_users<T: Retriever + Inserter>(State(state): State<Arc<App<T>>>) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    if let Ok(list) = db.get_users() {
        sb.push_str(r#"{"users":["#);
        for e in list.iter() {
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

async fn g_chats<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let se = String::from(r#"{"status":"Err"}"#);
    let Some(sid) = params.get("session_id") else {
        return se;
    };
    let Some(uid) = state.session_validate_str(sid) else {
        return se;
    };
    if let Ok(list) = db.get_chats(uid) {
        sb.push_str(r#"{"chats":["#);
        for e in &list {
            sb.push_str(
                format!(
                    r#"{{"chat_id":{},"title":"{}","description":"{}"}},"#,
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

async fn g_messages<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let se = String::from(r#"{"status":"Err"}"#);
    let Some(cid_str) = params.get("chat_id") else {
        return se;
    };
    let Ok(cid) = i64::from_str_radix(cid_str.as_str(), 10) else {
        return se;
    };
    if let Ok(list) = db.get_messages(cid) {
        sb.push_str(r#"{"messages":["#);
        for e in &list {
            sb.push_str(
                format!(
                    r#"{{"chat_id":{},"user_id":{},"content":"{}"}},"#,
                    e.chat_id, e.user_id, e.content
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

async fn g_devices<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> String {
    let mut sb = String::new();
    let db = state.storage.lock().unwrap();
    let se = String::from(r#"{"status":"Err"}"#);
    let Some(sid) = params.get("session_id") else {
        return se;
    };
    let Some(uid) = state.session_validate_str(sid) else {
        return se;
    };
    if let Ok(list) = db.get_devices(uid) {
        sb.push_str(r#"{"devices":["#);
        for e in &list {
            sb.push_str(
                format!(r#"{{"name":"{}","is_active":{}}},"#, e.name, e.is_active).as_str(),
            );
        }
        if !list.is_empty() {
            sb.pop();
        } // Remove trailing ','
        sb.push_str(r#"]}"#);
    }
    sb
}

async fn p_register(
    State(state): State<Arc<App<SQLite>>>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    if let (Some(name), Some(password)) = (payload["name"].as_str(), payload["password"].as_str()) {
        if let Some(()) = state.register(name, payload["surname"].as_str().unwrap_or("?"), password)
        {
            return r#"{"status":"Ok"}"#;
        }
    }
    r#"{"status":"Err"}"#
}

async fn p_login(
    State(state): State<Arc<App<SQLite>>>,
    Json(payload): Json<serde_json::Value>,
) -> String {
    if let (Some(id_str), Some(password)) =
        (payload["user_id"].as_str(), payload["password"].as_str())
    {
        if let Ok(id) = i64::from_str_radix(id_str, 10) {
            if let Some(session_id) = state.login(id, password) {
                return format!(r#"{{"status":"Ok", "session_id": {}}}"#, session_id);
            }
        }
    }
    String::from(r#"{"status":"Err"}"#)
}

async fn p_invite(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    if let (Some(sid), Some(target_str), Some(chat_id_str)) = (
        params.get("session_id"),
        payload["user_id"].as_str(),
        payload["chat_id"].as_str(),
    ) {
        let se = r#"{"status":"Err"}"#;
        let Some(_) = state.session_validate_str(sid) else {
            return se;
        };
        let Ok(target) = i64::from_str_radix(target_str, 10) else {
            return se;
        };
        let Ok(chat_id) = i64::from_str_radix(chat_id_str, 10) else {
            return se;
        };
        if let Some(()) = state.invite(target, chat_id) {
            return r#"{"status":"Ok"}"#;
        }
    }
    r#"{"status":"Err"}"#
}

async fn p_create(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    if let (Some(sid), Some(title), Some(description)) = (
        params.get("session_id"),
        payload["title"].as_str(),
        payload["description"].as_str(),
    ) {
        let se = r#"{"status":"Err"}"#;
        let Some(uid) = state.session_validate_str(sid) else {
            return se;
        };
        if let Some(chat_id) = state.make_chat(title, description) {
            state.invite(uid, chat_id);
            return r#"{"status":"Ok"}"#;
        }
    }
    r#"{"status":"Err"}"#
}

async fn p_message(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    if let (Some(sid), Some(chat_id_str), Some(content)) = (
        params.get("session_id"),
        payload["chat_id"].as_str(),
        payload["content"].as_str(),
    ) {
        let Some(uid) = state.session_validate_str(sid) else {
            return r#"{"status":"Err", "code":3}"#;
        };
        let Ok(chat_id) = i64::from_str_radix(chat_id_str, 10) else {
            return r#"{"status":"Err", "code":4}"#;
        };
        if let Some(()) = state.message(uid, chat_id, content) {
            return r#"{"status":"Ok"}"#;
        }
    }
    r#"{"status":"Err", "code":0}"#
}

#[tokio::main]
async fn main() {
    let app = Arc::new(App::new_debug());
    let router = Router::new()
        .route("/users", get(g_users::<SQLite>))
        .route("/chats", get(g_chats::<SQLite>))
        .route("/messages", get(g_messages::<SQLite>))
        .route("/devices", get(g_devices::<SQLite>))
        .route("/register", post(p_register))
        .route("/login", post(p_login))
        .route("/message", post(p_message))
        .route("/invite", post(p_invite))
        .route("/create", post(p_create))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
