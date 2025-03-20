use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct CompileError {
    pub msg: String,
}

impl CompileError {
    pub fn new(msg: String) -> Self {
        CompileError { msg }
    }
}
