//! axum server

use crate::{adapter::http::routes, config::SHARED_APP_STATE, error::AppError};

pub async fn start_http_server() -> Result<(), AppError> {
    let state = SHARED_APP_STATE.read().await.clone();
    let addr = format!("{}:{}", state.config.server.addr, state.config.server.port);
    tracing::info!("start server: {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, routes(state.clone()).await)
        .await
        .unwrap();
    tracing::info!("end server");
    Ok(())
}
