//! error

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AnyHowError(#[from] anyhow::Error),
}
