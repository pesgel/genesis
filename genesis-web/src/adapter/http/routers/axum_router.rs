use axum::routing::delete;
use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::adapter::http::handlers::*;
use crate::adapter::http::middleware::auth::jwt_auth_middle;
use crate::{adapter::http::handler_ssh, config::AppState};

pub async fn routes(state: AppState) -> Router {
    let open_route = Router::new()
        .route("/hello", get(hello))
        .route("/ssh", get(handler_ssh))
        .route("/login", post(user_login))
        .route("/register", post(user_register));

    let business = Router::new()
        .nest(
            "/instruct",
            Router::new()
                .route("/", post(save_instruct))
                .route("/:id", get(get_instruct_by_id))
                .route("/list", post(list_instruct))
                .route("/execute", post(execute_instruct))
                .route("/:id", delete(delete_instruct_by_id)),
        )
        .nest(
            "/node",
            Router::new()
                .route("/", post(save_node))
                .route("/:id", get(get_node_by_id))
                .route("/:id", delete(delete_node_by_id))
                .route("/kv", get(node_select_kv_item))
                .route("/list", post(list_node)),
        )
        .nest(
            "/execute",
            Router::new()
                .route("/:id", get(get_execute_by_id))
                .route("/stop/:id", get(stop_execute_by_id))
                .route("/list", post(list_execute))
                .route("/:id", delete(delete_execute_history_by_id)),
        )
        .nest(
            "/user",
            Router::new()
                .route("/info", get(user_info))
                .route("/:id", delete(delete_user_by_id)),
        )
        .layer(middleware::from_fn(jwt_auth_middle));

    Router::new()
        .nest("/api/v1", Router::new().merge(open_route).merge(business))
        .with_state(state)
}

// 处理函数
async fn hello() -> Json<serde_json::Value> {
    Json(json!({
        "version": "v0.0.1",
        "status": "running",
        "time": chrono::Local::now().to_string()
    }))
}
