use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErisError {
    #[error("Path parse failed {0}")]
    PathError(&'static str),
}

pub type ErisResult<T, E = ErisError> = anyhow::Result<T, E>;
