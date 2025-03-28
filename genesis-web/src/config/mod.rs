//! config

mod alias;

pub use alias::*;

use std::path::Path;

use crate::config::Db::Sqlite;
use crate::error::AppError;
use crate::util::jwt::JwtConfig;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
    #[serde(rename = "db")]
    pub db_config: Db,
    #[serde(rename = "jwt")]
    pub jwt_config: JwtConfig,
    #[serde(rename = "tracing")]
    pub tracing: Option<TracingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Db {
    #[serde(rename = "mysql")]
    Mysql(MysqlConfig),
    #[serde(rename = "sqlite")]
    Sqlite(SqliteConfig),
}

impl Default for Db {
    fn default() -> Self {
        Sqlite(SqliteConfig {
            path: "db.sqlite".to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub addr: String,
    pub port: String,
    #[serde(default = "genesis_common::_default_recording_path")]
    pub recording_path: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TracingConfig {
    pub filter: String,
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

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
}
impl SqliteConfig {
    pub fn connect_url(&self) -> String {
        format!("sqlite://{}", self.path)
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
    info!("config:{:?}", config);
    // build global config
    let mut init_config = SHARED_APP_CONFIG.write().await;
    *init_config = config.clone();
    Ok(config)
}
