use crate::adapter::http::handlers::*;
use crate::adapter::http::middleware::auth::jwt_auth_middle;
use crate::adapter::http::middleware::request_id::request_id;
use crate::adapter::http::middleware::server_time::ServerTimeLayer;
use crate::{adapter::http::handler_ssh, config::AppState};
use axum::routing::delete;
use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub async fn routes(state: AppState) -> Router {
    let open_route = Router::new()
        .route("/hello", get(hello))
        .route("/login", post(user_login))
        .route("/register", post(user_register))
        .layer(ServerTimeLayer);

    let business = Router::new()
        .nest(
            "/op",
            Router::new()
                .route("/ssh", get(handler_ssh))
                .route("/guacamole", get(handler_guacamole)),
        )
        .nest(
            "/instruct",
            Router::new()
                .route("/", post(save_instruct))
                .route("/list", post(list_instruct))
                .route("/execute", post(execute_instruct))
                .route(
                    "/:id",
                    get(get_instruct_by_id).delete(delete_instruct_by_id),
                ),
        )
        .nest(
            "/node",
            Router::new()
                .route("/", post(save_node))
                .route("/:id", get(get_node_by_id).delete(delete_node_by_id))
                .route("/kv", get(node_select_kv_item))
                .route("/list", post(list_node)),
        )
        .nest(
            "/asset",
            Router::new()
                .route("/:id", get(get_asset_by_id).delete(delete_asset_by_id))
                .route("/", post(save_asset))
                .route("/protocol/list", post(list_asset_protocol))
                .route("/list", post(list_asset)),
            //list_asset_protocol
        )
        .nest(
            "/credential",
            Router::new()
                .route("/", post(save_credential))
                .route(
                    "/:id",
                    get(get_credential_by_id).delete(delete_credential_by_id),
                )
                .route("/list", post(list_asset_credential)),
        )
        .nest(
            "/execute",
            Router::new()
                .route(
                    "/:id",
                    get(get_execute_by_id).delete(delete_execute_history_by_id),
                )
                .route("/stop/:id", get(stop_execute_by_id))
                .route("/list", post(list_execute))
                .route("/recording/download/:id", get(execute_recording)),
        )
        .nest(
            "/user",
            Router::new()
                .route("/info", get(user_info))
                .route("/:id", delete(delete_user_by_id)),
        )
        .layer(middleware::from_fn(jwt_auth_middle))
        .layer(middleware::from_fn(request_id))
        .layer(ServerTimeLayer);

    Router::new()
        .nest("/api/v1", Router::new().merge(open_route).merge(business))
        // 静态文件服务
        // .nest_service("/", get_service(ServeDir::new("./static")))
        // 捕获所有未匹配路由返回前端入口
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
