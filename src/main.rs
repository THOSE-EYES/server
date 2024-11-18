use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::string::String;
use std::sync::Arc;

mod app;
mod auth;
mod db;
mod utils;

use app::App;
use db::{drivers::SQLite, Inserter, Retriever};

/// [handler] GET /users
///
/// Returns: {schema}
async fn g_users<T: Retriever + Inserter>(State(state): State<Arc<App<T>>>) -> Response {
    let db = state.storage.lock().unwrap();
    if let Ok(list) = db.get_users() {
        (StatusCode::OK, Json(json!({"users": list}))).into_response()
    } else {
        (StatusCode::NOT_FOUND).into_response()
    }
}

/// [handler] GET /chats
///
/// Returns: {schema}
async fn g_chats<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let db = state.storage.lock().unwrap();
    let Some(sid) = params.get("session_id") else {
        return (StatusCode::BAD_REQUEST).into_response();
    };
    let Some(uid) = state.session_validate_str(sid) else {
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    if let Ok(list) = db.get_chats(uid) {
        return (StatusCode::OK, Json(json!({"chats": list}))).into_response();
    }
    (StatusCode::NOT_FOUND).into_response()
}

/// [handler] GET /messages
///
/// Returns: {schema}
async fn g_messages_sec<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let db = state.storage.lock().unwrap();
    let Some(sid_str) = params.get("session_id") else {
        return (StatusCode::BAD_REQUEST).into_response();
    };
    let Some(uid) = state.session_validate_str(sid_str) else {
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    let Some(cid) = payload["chat_id"].as_i64() else {
        return (StatusCode::BAD_REQUEST).into_response();
    };
    let Ok(chats) = db.get_chats(uid) else {
        return (StatusCode::NOT_FOUND).into_response();
    };
    let Some(_) = chats.iter().find(|e| e.id == cid) else {
        return (StatusCode::NOT_FOUND).into_response();
    };
    if let Ok(list) = db.get_messages(cid) {
        return (StatusCode::OK, Json(json!({"messages": list}))).into_response();
    }
    (StatusCode::BAD_REQUEST).into_response()
}

/// [handler] GET /devices
///
/// Returns: {schema}
async fn g_devices<T: Retriever + Inserter>(
    State(state): State<Arc<App<T>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let db = state.storage.lock().unwrap();
    let Some(sid) = params.get("session_id") else {
        return (StatusCode::BAD_REQUEST).into_response();
    };
    let Some(uid) = state.session_validate_str(sid) else {
        return (StatusCode::BAD_REQUEST).into_response();
    };
    if let Ok(list) = db.get_devices(uid) {
        return (StatusCode::OK, Json(json!({"devices": list}))).into_response();
    }
    (StatusCode::BAD_REQUEST).into_response()
}

/// [handler] POST /register
///
/// Returns: {schema}
async fn p_register<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(name), Some(password)) = (payload["name"].as_str(), payload["password"].as_str()) {
        if let Some(id) = state.register(name, payload["surname"].as_str().unwrap_or("?"), password)
        {
            return (StatusCode::OK, Json(json!({"user_id": id}))).into_response();
        }
    }
    return (StatusCode::BAD_REQUEST).into_response();
}

/// [handler] POST /register
///
/// Returns: {schema}
async fn p_login<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(id), Some(password)) = (payload["user_id"].as_i64(), payload["password"].as_str())
    {
        if let Some(session_id) = state.login(id, password) {
            return (
                StatusCode::OK,
                Json(json!({"session_id": session_id, "user_id": id})),
            )
                .into_response();
        }
    }
    (StatusCode::UNAUTHORIZED).into_response()
}

async fn g_active_sec<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(sid_str), Some(id)) = (params.get("session_id"), payload["user_id"].as_i64()) {
        let Some(_) = state.session_validate_str(sid_str) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        let Ok(_) = i64::from_str_radix(sid_str, 10) else {
            return (StatusCode::BAD_REQUEST).into_response();
        };
        if let Some(b) = state.is_active(id) {
            return (StatusCode::OK, Json(json!({"active": b}))).into_response();
        }
    }
    (StatusCode::BAD_REQUEST).into_response()
}

/// [handler] POST /invite
///
/// Returns: {schema}
async fn p_invite<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(sid), Some(target), Some(chat_id)) = (
        params.get("session_id"),
        payload["user_id"].as_i64(),
        payload["chat_id"].as_i64(),
    ) {
        let Some(_) = state.session_validate_str(sid) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        if let Some(()) = state.invite(target, chat_id) {
            return (StatusCode::OK).into_response();
        }
    }
    (StatusCode::BAD_REQUEST).into_response()
}

/// [handler] POST /create
///
/// Returns: {schema}
async fn p_create<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(sid), Some(title), Some(description)) = (
        params.get("session_id"),
        payload["title"].as_str(),
        payload["description"].as_str(),
    ) {
        let Some(uid) = state.session_validate_str(sid) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        if let Some(chat_id) = state.create_chat(title, description) {
            state.invite(uid, chat_id);
            return (StatusCode::OK).into_response();
        }
    }
    (StatusCode::BAD_REQUEST).into_response()
}

async fn p_logout<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Some(sid_str) = params.get("session_id") {
        let Some(_) = state.session_validate_str(sid_str) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        let Ok(sid) = i64::from_str_radix(sid_str, 10) else {
            return (StatusCode::BAD_REQUEST).into_response();
        };
        if let Some(_) = state.logout(sid) {
            return (StatusCode::OK).into_response();
        }
    }
    return (StatusCode::BAD_REQUEST).into_response();
}

/// [handler] POST /message
///
/// Returns: {schema}
async fn p_message<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    if let (Some(sid), Some(chat_id), Some(content)) = (
        params.get("session_id"),
        payload["chat_id"].as_i64(),
        payload["content"].as_str(),
    ) {
        let Some(uid) = state.session_validate_str(sid) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        if let Some(()) = state.message(uid, chat_id, content) {
            return (StatusCode::OK).into_response();
        }
    }
    (StatusCode::BAD_REQUEST).into_response()
}

async fn p_heartbeat<T: Retriever + Inserter>(
    State(state): State<Arc<App<SQLite>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Some(sid) = params.get("session_id") {
        let Some(uid) = state.session_validate_str(sid) else {
            return (StatusCode::UNAUTHORIZED).into_response();
        };
        if let Some(()) = state.set_activity(uid) {
            return (StatusCode::OK).into_response();
        }
    }
    (StatusCode::BAD_REQUEST).into_response()
}

#[tokio::main]
async fn main() {
    let app = Arc::new(App::new_debug());

    // Start the reaper thread which checks if heartbeats are sent
    let clone = app.clone();
    let _thread = tokio::task::spawn(async move {
        clone.reaper();
    });

    let router = Router::new()
        .route("/users", get(g_users::<SQLite>))
        .route("/getUsers", get(g_users::<SQLite>))
        .route("/chats", get(g_chats::<SQLite>))
        .route("/messages", get(g_messages_sec::<SQLite>))
        .route("/messages", post(g_messages_sec::<SQLite>))
        .route("/devices", get(g_devices::<SQLite>))
        .route("/register", post(p_register::<SQLite>))
        .route("/login", post(p_login::<SQLite>))
        .route("/logout", get(p_logout::<SQLite>))
        .route("/logout", post(p_logout::<SQLite>))
        .route("/message", post(p_message::<SQLite>))
        .route("/invite", post(p_invite::<SQLite>))
        .route("/create", post(p_create::<SQLite>))
        .route("/heartbeat", post(p_heartbeat::<SQLite>))
        .route("/sendActivity", post(p_heartbeat::<SQLite>))
        .route("/getActivity", get(g_active_sec::<SQLite>))
        .route("/getActivity", post(g_active_sec::<SQLite>))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
