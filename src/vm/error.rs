use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct VmError {
    pub msg: String,
}

impl VmError {
    pub fn new(msg: String) -> Self {
        VmError { msg }
    }
}
