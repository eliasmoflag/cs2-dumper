#![allow(dead_code)]

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("not attached")]
    NotAttached,
    #[error("already attached")]
    AlreadyAttached,

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    WindowsError(#[from] windows::core::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    PeliteError(#[from] pelite::Error),
}
