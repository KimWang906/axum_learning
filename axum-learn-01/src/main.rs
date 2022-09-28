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
        .route("/user", post(create_user)) // route: "/user" 경우, porst(create_user)를 POST 방식으로 전송
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
