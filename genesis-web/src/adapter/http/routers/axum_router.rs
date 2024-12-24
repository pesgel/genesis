use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::adapter::http::handlers::*;
use crate::adapter::http::middleware::auth::jwt_auth_middle;
use crate::{
    adapter::http::{handler_ssh, handlers::get_instruct_by_id},
    config::AppState,
};

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
                .route("/execute/:id", post(execute_instruct_by_id)),
        )
        .nest(
            "/node",
            Router::new()
                .route("/", post(save_node))
                .route("/:id", get(get_node_by_id))
                .route("/list", post(list_node)),
        )
        .nest("/user", Router::new().route("/info", get(user_info)))
        .layer(middleware::from_fn(jwt_auth_middle));

    Router::new()
        .nest("/api/v1", Router::new().merge(open_route).merge(business))
        .with_state(state)
}

// 处理函数
async fn hello() -> &'static str {
    "Hello, World!"
}
