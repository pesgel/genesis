use sea_orm::DatabaseConnection;

pub mod adapter;
pub mod cmd;
pub mod common;
pub mod config;
pub mod error;
pub mod repo;
pub mod service;
pub mod util;
#[derive(Clone, Default)]
pub struct AppState {
    pub conn: DatabaseConnection,
}
