use axum::{routing::get, Router};

use crate::{adapter::http::handler_ssh, config::AppState};

pub async fn routes(state: AppState) -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/ssh", get(handler_ssh))
        .with_state(state)
}

// 处理函数
async fn hello() -> &'static str {
    "Hello, World!"
}
