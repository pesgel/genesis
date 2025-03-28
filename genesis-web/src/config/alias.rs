//! runtime pram

use super::{AppConfig, Db};
use lazy_static::lazy_static;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::collections::HashMap;
use tokio::sync::{watch, RwLock};

lazy_static! {
    pub static ref SHARED_APP_STATE: RwLock<AppState> = RwLock::new(AppState::default());
    pub static ref SHARED_APP_CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default());
    pub static ref EXECUTE_MAP_MANAGER: RwLock<HashMap<String, watch::Sender<bool>>> =
        RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub conn: DatabaseConnection,
}
pub async fn init_shared_app_state(config: &AppConfig) -> Result<AppState, ()> {
    let mut state = AppState {
        conn: DatabaseConnection::Disconnected,
    };
    // step1. db connect
    state.conn = db_init(config)
        .await
        .map_err(|e| format!("init db connect error: {:?}", e))
        .unwrap();
    let mut sas = SHARED_APP_STATE.write().await;
    *sas = state.clone();
    tracing::debug!("app state initialized");
    Ok(state)
}

async fn db_init(config: &AppConfig) -> Result<DatabaseConnection, DbErr> {
    match config.db_config.clone() {
        Db::Mysql(conf) => Database::connect(conf.connect_url()).await,
        Db::Sqlite(conf) => Database::connect(conf.connect_url()).await,
    }
}
