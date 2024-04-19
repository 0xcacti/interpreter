use crate::{
    code::Instructions,
    evaluator::object::{self, Object},
};

use super::error::VmError;

pub struct Frame {
    pub function: Object,
    pub ip: usize,
}

impl Frame {
    pub fn new(function: object::Object) -> Result<Frame, VmError> {
        match function {
            Object::CompiledFunction(_) => Ok(Frame { function, ip: 0 }),
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                function
            ))),
        }
    }

    pub fn instructions(&self) -> Result<Instructions, VmError> {
        match &self.function {
            Object::CompiledFunction(function) => Ok(function.clone()),
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                self.function
            ))),
        }
    }
}
