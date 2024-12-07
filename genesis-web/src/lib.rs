use sea_orm::DatabaseConnection;

pub mod adapter;
pub mod cmd;
pub mod common;
pub mod config;
pub mod error;

#[derive(Clone, Default)]
pub struct AppState {
    pub conn: DatabaseConnection,
}
