#![allow(dead_code)]

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("not attached")]
    NotAttached,
    #[error("already attached")]
    AlreadyAttached,
    #[error("invalid image")]
    InvalidImage,

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[cfg(target_os = "windows")]
    #[error(transparent)]
    PeliteError(#[from] pelite::Error),

    #[cfg(target_os = "windows")]
    #[error(transparent)]
    WindowsError(#[from] windows::core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
