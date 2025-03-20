use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct CodeError {
    pub msg: String,
}

impl CodeError {
    pub fn new(msg: String) -> Self {
        CodeError { msg }
    }
}
