use sea_orm::DatabaseConnection;

pub mod adapter;
pub mod cmd;
pub mod common;
pub mod config;
pub mod error;
pub mod repo;

#[derive(Clone, Default)]
pub struct AppState {
    pub conn: DatabaseConnection,
}
