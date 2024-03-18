use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct VmError {
    pub msg: &str,
}

impl VmError {
    pub fn new(msg: &str) -> VmError {
        VmError { msg }
    }
}
