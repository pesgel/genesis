//! runtime pram

use super::AppConfig;
use lazy_static::lazy_static;
use sea_orm::{Database, DatabaseConnection};
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
    let mut state = AppState::default();
    // step1. 构造mysql
    match Database::connect(config.mysql_config.connect_url()).await {
        Ok(conn) => {
            state.conn = conn;
            tracing::debug!("mysql conn initialized");
        }
        Err(e) => {
            tracing::error!("create db conn error: {:?}", e);
            return Err(());
        }
    }
    let mut sas = SHARED_APP_STATE.write().await;
    *sas = state.clone();
    tracing::debug!("app state initialized");
    Ok(state)
}
