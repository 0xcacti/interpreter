use crate::evaluator::error::EvaluatorError;
use std::fmt;
use std::rc::Rc;

use super::object::Object;

#[derive(Debug, PartialEq, Clone)]
pub enum Builtin {
    Len,
    First,
    Last,
    Rest,
    Push,
    Echo,
    Echoln,
}

impl Builtin {
    pub fn lookup(name: &str) -> Option<Object> {
        match name {
            "len" => Some(Object::Builtin(Builtin::Len)),
            "first" => Some(Object::Builtin(Builtin::First)),
            "last" => Some(Object::Builtin(Builtin::Last)),
            "rest" => Some(Object::Builtin(Builtin::Rest)),
            "push" => Some(Object::Builtin(Builtin::Push)),
            "echo" => Some(Object::Builtin(Builtin::Echo)),
            "echoln" => Some(Object::Builtin(Builtin::Echoln)),
            _ => None,
        }
    }
    pub fn apply(&self, args: &Vec<Rc<Object>>) -> Result<Rc<Object>, EvaluatorError> {
        match self {
            Builtin::Len => {
                check_argument_count(1, args.len())?;
                match *args[0] {
                    Object::String(ref s) => Ok(Rc::new(Object::Integer(s.len() as i64))),
                    Object::Array(ref a) => Ok(Rc::new(Object::Integer(a.len() as i64))),
                    _ => Err(EvaluatorError::new(format!(
                        "argument to `len` not supported, got {}",
                        args[0]
                    ))),
                }
            }
            _ => Err(EvaluatorError::new(format!(
                "builtin not implemented: {}",
                self
            ))),
            // Builtin::First => first(args),
            // Builtin::Last => last(args),
            // Builtin::Rest => rest(args),
            // Builtin::Push => push(args),
            // Builtin::Echo => echo(args),
            // Builtin::Echoln => echoln(args),
        }
    }
}

fn check_argument_count(expected: usize, actual: usize) -> Result<(), EvaluatorError> {
    if expected != actual {
        Err(EvaluatorError::new(format!(
            "wrong number of arguments. expected={}, got={}",
            expected, actual
        )))
    } else {
        Ok(())
    }
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Builtin::Len => write!(f, "len"),
            Builtin::First => write!(f, "first"),
            Builtin::Last => write!(f, "last"),
            Builtin::Rest => write!(f, "rest"),
            Builtin::Push => write!(f, "push"),
            Builtin::Echo => write!(f, "echo"),
            Builtin::Echoln => write!(f, "echoln"),
        }
    }
}
