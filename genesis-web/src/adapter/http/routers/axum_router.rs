use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    adapter::http::{
        handler_ssh,
        handlers::{get_instruct_by_id, save_and_execute},
    },
    config::AppState,
};

pub async fn routes(state: AppState) -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/ssh", get(handler_ssh))
        .nest(
            "/api/instruct",
            Router::new()
                .route("/:id", get(get_instruct_by_id))
                .route("/", post(save_and_execute)),
        )
        .with_state(state)
}

// 处理函数
async fn hello() -> &'static str {
    "Hello, World!"
}
