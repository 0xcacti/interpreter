use std::fmt;
use std::rc::Rc;

use super::error::ObjectError;
use super::Object;

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum Builtin {
    Len,
    First,
    Last,
    Rest,
    Push,
    Echo,
    Echoln,
}

impl From<u8> for Builtin {
    fn from(op: u8) -> Builtin {
        match op {
            0 => Builtin::Len,
            1 => Builtin::First,
            2 => Builtin::Last,
            3 => Builtin::Rest,
            4 => Builtin::Push,
            5 => Builtin::Echo,
            6 => Builtin::Echoln,
            _ => panic!("unknown builtin index"),
        }
    }
}

impl Builtin {
    pub fn variants() -> Vec<&'static str> {
        vec!["len", "first", "last", "rest", "push", "echo", "echoln"]
    }

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
    pub fn apply(&self, args: &Vec<Rc<Object>>) -> Result<Rc<Object>, ObjectError> {
        match self {
            Builtin::Len => {
                check_argument_count(1, args.len())?;
                match *args[0] {
                    Object::String(ref s) => Ok(Rc::new(Object::Integer(s.len() as i64))),
                    Object::Array(ref a) => Ok(Rc::new(Object::Integer(a.len() as i64))),
                    _ => Err(ObjectError::new(format!(
                        "argument to `len` not supported, got {}",
                        args[0]
                    ))),
                }
            }
            Builtin::First => {
                check_argument_count(1, args.len())?;
                match *args[0] {
                    Object::Array(ref a) => {
                        if a.len() > 0 {
                            Ok(a[0].clone())
                        } else {
                            Ok(Rc::new(Object::Null))
                        }
                    }
                    _ => Err(ObjectError::new(format!(
                        "argument to `first` must be ARRAY, got {}",
                        args[0]
                    ))),
                }
            }

            Builtin::Last => {
                check_argument_count(1, args.len())?;
                match *args[0] {
                    Object::Array(ref a) => {
                        if a.len() > 0 {
                            Ok(a[a.len() - 1].clone())
                        } else {
                            Ok(Rc::new(Object::Null))
                        }
                    }
                    _ => Err(ObjectError::new(format!(
                        "argument to `last` must be ARRAY, got {}",
                        args[0]
                    ))),
                }
            }
            Builtin::Rest => {
                check_argument_count(1, args.len())?;
                match *args[0] {
                    Object::Array(ref a) => {
                        if a.len() > 0 {
                            let mut new_array = Vec::new();
                            for i in 1..a.len() {
                                new_array.push(a[i].clone());
                            }
                            Ok(Rc::new(Object::Array(new_array)))
                        } else {
                            Ok(Rc::new(Object::Null))
                        }
                    }
                    _ => Err(ObjectError::new(format!(
                        "argument to `rest` must be ARRAY, got {}",
                        args[0]
                    ))),
                }
            }
            Builtin::Push => {
                check_argument_count(2, args.len())?;
                match *args[0] {
                    Object::Array(ref a) => {
                        let mut new_array = Vec::new();
                        for i in 0..a.len() {
                            new_array.push(a[i].clone());
                        }
                        new_array.push(args[1].clone());
                        Ok(Rc::new(Object::Array(new_array)))
                    }
                    _ => Err(ObjectError::new(format!(
                        "argument to `push` must be ARRAY, got {}",
                        args[0]
                    ))),
                }
            }
            Builtin::Echo => {
                for arg in args {
                    print!("{}", arg);
                }

                Ok(Rc::new(Object::Null))
            }
            Builtin::Echoln => {
                for arg in args {
                    print!("{}", arg);
                }
                println!();
                Ok(Rc::new(Object::Null))
            }
        }
    }
}

fn check_argument_count(expected: usize, actual: usize) -> Result<(), ObjectError> {
    if expected != actual {
        Err(ObjectError::new(format!(
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
