use std::{
    fmt::{Display, Formatter, Result},
    rc::Rc,
};

use crate::parser::ast::Statement;

use super::environment::Env;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(String),
    ReturnValue(Rc<Object>),
    Function(Vec<String>, Vec<Statement>, Env),
    Null,
}

impl Object {
    pub fn is_integer(&self) -> bool {
        match self {
            Object::Integer(_) => true,
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::ReturnValue(o) => write!(f, "{}", o),
            Object::Null => write!(f, "null"),
            Object::Function(parameters, _, _) => {
                let params = parameters.join(", ");
                write!(f, "fn({}) {{...}}", params)
            }
        }
    }
}

impl From<bool> for Object {
    fn from(b: bool) -> Self {
        Object::Boolean(b)
    }
}

impl From<i64> for Object {
    fn from(i: i64) -> Self {
        Object::Integer(i)
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Object::String(s)
    }
}
