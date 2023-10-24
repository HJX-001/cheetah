use notify_debouncer_full::notify;
use thiserror::Error;

pub type WatchResult<T> = std::result::Result<T, WatchError>;

#[derive(Debug, Error)]
pub enum WatchError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    NotifyError(#[from] notify::Error),
    #[error(transparent)]
    SendError(#[from] std::sync::mpsc::SendError<String>),
    #[error("watcher with uid `{0}` already registered ")]
    DuplicateUid(usize),
    #[error("watcher with uid `{0}` already unregistered or not exists")]
    UidNotFound(usize),
    #[error("pattern `{0}` is not valid glob pattern, error: {1}")]
    PatternError(String, glob::PatternError),
    #[error("[experimental]pattern `{0}` is not valid glob pattern")]
    ExPatternError(String),
}
