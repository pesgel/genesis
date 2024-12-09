//! config

mod alias;

pub use alias::*;

use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
    #[serde(rename = "mysql")]
    pub mysql_config: MysqlConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerConfig {
    pub addr: String,
    pub port: String,
}
impl ServerConfig {
    pub fn url(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}
#[derive(Debug, Default, Clone, Deserialize)]
pub struct MysqlConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}
impl MysqlConfig {
    pub fn connect_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}/{}",
            self.username, self.password, self.host, self.database
        )
    }
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
    let mut init_config = SHARED_APP_CONFIG.write().await;
    *init_config = config.clone();
    Ok(config)
}
