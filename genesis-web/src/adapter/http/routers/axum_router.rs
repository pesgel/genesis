use axum::{
    routing::{get, post},
    Router,
};

use crate::adapter::http::handlers::{execute_instruct_by_id, list_instruct, save_instruct};
use crate::{
    adapter::http::{handler_ssh, handlers::get_instruct_by_id},
    config::AppState,
};

pub async fn routes(state: AppState) -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/ssh", get(handler_ssh))
        .nest(
            "/api/instruct",
            Router::new()
                .route("/", post(save_instruct))
                .route("/:id", get(get_instruct_by_id))
                .route("/list", post(list_instruct))
                .route("/execute/:id", post(execute_instruct_by_id)),
        )
        .with_state(state)
}

// 处理函数
async fn hello() -> &'static str {
    "Hello, World!"
}
