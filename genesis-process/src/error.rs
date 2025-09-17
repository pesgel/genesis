use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // #[error("{0}")]
    // MsgError(String),
    // #[error(transparent)]
    // DbError(#[from] DbErr),
    // #[error(transparent)]
    // AnyHowError(#[from] anyhow::Error),
    // #[error(transparent)]
    // OtherError(#[from] Box<dyn std::error::Error>),
    // #[error(transparent)]
    // AuthError(#[from] AuthError),
    // #[error(transparent)]
    // SerdeJsonError(#[from] serde_json::Error),
}
