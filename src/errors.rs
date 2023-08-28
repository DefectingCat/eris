use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErisError {
    #[error("Target is empty {0}")]
    Empty(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

pub type ErisResult<T, E = ErisError> = anyhow::Result<T, E>;
