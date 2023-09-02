use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct EvaluatorError {
    pub msg: String,
}

impl EvaluatorError {
    pub fn new(msg: String) -> Self {
        EvaluatorError { msg }
    }
}
