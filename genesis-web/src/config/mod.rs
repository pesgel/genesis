//! config

mod alias;

pub use alias::*;

use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerConfig {
    pub addr: String,
    pub port: u16,
}

// parse config
pub async fn parse_config(path: &Path) -> Result<AppConfig, AppError> {
    // file path
    tracing::debug!("parse config from path: {:}", path.display());
    // read config data
    let data = std::fs::read_to_string(path).unwrap();
    // convert
    let config: AppConfig = toml::from_str(&data).unwrap();
    // build global config
    let mut state = SHARED_APP_STATE.write().await;
    state.config = config.clone();
    Ok(config)
}
