//! axum server

use crate::config::{AppConfig, AppState};
use crate::{adapter::http::routes, error::AppError};

pub async fn start_http_server(config: &AppConfig, state: AppState) -> Result<(), AppError> {
    let url = config.server.url();
    tracing::info!("start server: {}", url);
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();
    axum::serve(listener, routes(state).await).await.unwrap();
    tracing::info!("end server");
    Ok(())
}
