use thiserror::Error;

use crate::object::error::ObjectError;

#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum EvaluatorError {
    #[error("Evaluator error: {0}")]
    Native(String),
    #[error("Object error: {0}")]
    Object(#[from] ObjectError),
}

impl EvaluatorError {
    pub fn new(msg: String) -> Self {
        EvaluatorError::Native(msg)
    }
}
