//! runtime pram

use super::AppConfig;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref SHARED_APP_STATE: RwLock<AppState> = RwLock::new(AppState::default());
}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub config: AppConfig,
}
