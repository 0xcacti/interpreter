use crate::{code::Instructions, object::Object};
use std::rc::Rc;

use super::error::VmError;

#[derive(Debug, Clone)]
pub struct Frame {
    pub function: Rc<Object>,
    pub ip: isize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(function: Rc<Object>, base_pointer: usize) -> Result<Frame, VmError> {
        match &*function {
            Object::Closure(compiled_function, num_free) => Ok(Frame {
                function: Rc::new(Object::Closure(
                    compiled_function.clone(),
                    num_free.to_vec(),
                )),
                ip: -1,
                base_pointer,
            }),
            _ => Err(VmError::new(format!(
                "Expected Closure, got {:?}",
                function
            ))),
        }
    }

    pub fn instructions(&self) -> Result<Instructions, VmError> {
        match &*self.function {
            Object::Closure(compiled_function, _) => Ok(compiled_function.instructions().clone()),
            _ => Err(VmError::new(format!(
                "Expected Closure, got {:?}",
                self.function
            ))),
        }
    }
}
