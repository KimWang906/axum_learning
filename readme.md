# Axum 배우기

[axum-learn01 참고 링크](https://carlosmv.hashnode.dev/getting-started-with-axum-rust)

## Code

### JSON 요청, 응답

```rs
use axum::{
    routing::{get, post, get_service},
    http::StatusCode,
    response::IntoResponse,
    Json, Router};
use serde_json::json;
use tower_http::services::ServeFile;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use axum::extract::Path; 
use std::io;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Debug, Serialize,Deserialize, Clone, Eq, Hash, PartialEq)]
struct User {
    id: u64,
    username: String,
}

async fn create_user(Json(payload): Json<CreateUser>,) -> impl IntoResponse { // JSON 형식으로 user 생성
    let user = User {
        id: 1337,
        username: payload.username
    };

    (StatusCode::CREATED, Json(user))
}

async fn json_hello(Path(name): Path<String>) -> impl IntoResponse { // /hello/:name에 들어갈 시 JSON 메세지 보내기
    let greeting = name.as_str();
    let hello = String::from("Hello ");

    (StatusCode::OK, Json(json!({"message": hello + greeting })))
}

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(root)) // route(): "/" 경우, get(root)를 GET 방식으로 전송
        .route("/hello/:name", get(json_hello)) // route: "/hello/:name" :name은 고정적인 경로가 아님, get(json_hello)를 GET 방식으로 전송
        .route("/static", get_service(ServeFile::new("static/index.html"))
                .handle_error(|error: io::Error| async move { // Error Handling
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                }));

    

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // localhost
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str { // root -> "/", Hello World 출력
    "Hello, World!"
}
```

### WebSocket Learning

```rs
//! Example websocket server.
//!
//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-websockets
//! ```x

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::new()
        .fallback(
            get_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
                .handle_error(|error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                }),
        )
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("client sent str: {:?}", t);
                }
                Message::Binary(_) => {
                    println!("client sent binary data");
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(String::from("Hi!"))) // Client가 접속 시 Server가 메세지를 보냄
            .await
            .is_err() // Error Check
        {
            println!("client disconnected"); // 연결이 끊어졌음을 알림
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await; // 3초간 대기
    }
}
```

### Chat system with rust

