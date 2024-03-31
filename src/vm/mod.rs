pub mod error;

use crate::{
    code::{self, Instructions, Opcode},
    compiler,
    evaluator::object::{self, Object},
};
use error::VmError;

use std::rc::Rc;

const STACK_SIZE: usize = 2048;

pub struct VM {
    pub constants: Vec<Rc<Object>>,
    pub instructions: code::Instructions,
    pub stack: Vec<Rc<Object>>,
    pub sp: usize,
}

impl VM {
    pub fn new(bytecode: compiler::Bytecode) -> Self {
        return VM {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: vec![Rc::new(Object::Null); STACK_SIZE],
            sp: 0,
        };
    }

    pub fn stack_top(&self) -> Option<Rc<object::Object>> {
        if self.sp == 0 {
            return None;
        }
        Some(Rc::clone(&self.stack[self.sp - 1]))
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        let mut ip = 0;
        let instructions_len = self.instructions.len();

        while ip < instructions_len {
            let opcode = self.instructions[ip];

            match opcode.into() {
                Opcode::Constant => {
                    let constant_index = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip += 2;

                    if constant_index > self.constants.len() {
                        return Err(VmError::new("Invalid constant index".to_string()));
                    }
                    let constant = Rc::clone(&self.constants[constant_index]);
                    self.push(constant);
                }

                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div => {
                    self.execute_binary_instruction(opcode.into())?;
                }

                Opcode::Pop => {
                    self.pop();
                }

                Opcode::True => {
                    self.push(Rc::new(Object::Boolean(true)));
                }

                Opcode::False => {
                    self.push(Rc::new(Object::Boolean(false)));
                }

                Opcode::Equal | Opcode::NotEqual | Opcode::GreaterThan => {
                    self.execute_comparison(opcode.into())?;
                }
                _ => {
                    return Err(VmError::new("Invalid opcode".to_string()));
                }
            }
            ip += 1;
        }
        Ok(())
    }

    pub fn push(&mut self, obj: Rc<Object>) {
        if self.sp >= STACK_SIZE {
            panic!("stack overflow");
        }
        self.stack[self.sp] = obj;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> Rc<Object> {
        if self.sp == 0 {
            panic!("stack underflow");
        }
        let obj = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        obj
    }

    pub fn last_popped_stack_elem(&self) -> Rc<Object> {
        Rc::clone(&self.stack[self.sp])
    }

    pub fn execute_binary_instruction(&mut self, opcode: Opcode) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();

        match (&*left, &*right) {
            (Object::Integer(left), Object::Integer(right)) => {
                let result = match opcode {
                    Opcode::Add => left + right,
                    Opcode::Sub => left - right,
                    Opcode::Mul => left * right,
                    Opcode::Div => left / right,
                    _ => return Err(VmError::new("Invalid opcode".to_string())),
                };
                self.push(Rc::new(Object::Integer(result)));
            }
            _ => {
                return Err(VmError::new(
                    "Unsupported types for binary operation".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn execute_comparison(&mut self, opcode: Opcode) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();
        match (&*left, &*right) {
            (Object::Integer(left), Object::Integer(right)) => {
                return self.execute_integer_comparison(opcode, *left, *right);
            }
            _ => match opcode {
                Opcode::Equal => {
                    let result = Rc::new(Object::Boolean(left == right));
                    self.push(result);
                }
                Opcode::NotEqual => {
                    let result = Rc::new(Object::Boolean(left != right));
                    self.push(result);
                }
                _ => {
                    return Err(VmError::new(
                        "Unsupported comparison operation for type".to_string(),
                    ));
                }
            },
        }

        Ok(())
    }

    pub fn execute_integer_comparison(
        &mut self,
        opcode: Opcode,
        left: i64,
        right: i64,
    ) -> Result<(), VmError> {
        let result = match opcode {
            Opcode::Equal => left == right,
            Opcode::NotEqual => left != right,
            Opcode::GreaterThan => left > right,
            _ => {
                return Err(VmError::new("Invalid opcode".to_string()));
            }
        };
        self.push(Rc::new(Object::Boolean(result)));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use super::*;
    use crate::{
        compiler::Compiler,
        lexer::Lexer,
        parser::{ast, Parser},
    };

    struct VmTest {
        input: String,
        expected: object::Object,
    }

    fn parse(input: &str) -> ast::Node {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        ast::Node::Program(parser.parse_program().unwrap())
    }

    fn run_vm_tests(tests: Vec<VmTest>) {
        for test in tests {
            let program = parse(&test.input);
            let mut comp = Compiler::new();
            comp.compile(program).unwrap();

            let mut vm = VM::new(comp.bytecode());
            vm.run().unwrap();

            let last = vm.last_popped_stack_elem();

            test_expected_object(test.expected, last.clone().deref().clone());
        }
    }

    fn validate_integer_object(obj: object::Object, expected: i64) {
        match obj {
            Object::Integer(value) => assert_eq!(value, expected),
            _ => panic!("object not integer"),
        }
    }

    fn validate_boolean_object(obj: object::Object, expected: bool) {
        match obj {
            Object::Boolean(value) => assert_eq!(value, expected),
            _ => panic!("object not boolean"),
        }
    }

    fn test_expected_object(expected: object::Object, actual: object::Object) {
        match expected {
            Object::Integer(expected) => validate_integer_object(actual, expected),
            Object::Boolean(expected) => validate_boolean_object(actual, expected),
            _ => panic!("unsupported object type"),
        }
    }

    #[test]
    fn it_adds_two_integers() {
        let tests = vec![VmTest {
            input: "1 + 2".to_string(),
            expected: object::Object::Integer(3),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_subtracts_two_integers() {
        let tests = vec![VmTest {
            input: "2 - 1".to_string(),
            expected: object::Object::Integer(1),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_multiplies_two_integers() {
        let tests = vec![VmTest {
            input: "2 * 2".to_string(),
            expected: object::Object::Integer(4),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_divides_two_integers() {
        let tests = vec![VmTest {
            input: "4 / 2".to_string(),
            expected: object::Object::Integer(2),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_pushes_bools() {
        let tests = vec![
            VmTest {
                input: "true".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "false".to_string(),
                expected: object::Object::Boolean(false),
            },
        ];
        run_vm_tests(tests);
    }
    #[test]
    fn it_compares() {
        let tests = vec![
            VmTest {
                input: "1 < 2".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "1 > 2".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "1 < 1".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "1 > 1".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "1 == 1".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "1 != 1".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "1 == 2".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "1 != 2".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "true == true".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "false == false".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "true == false".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "true != false".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "false != true".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "(1 < 2) == true".to_string(),
                expected: object::Object::Boolean(true),
            },
            VmTest {
                input: "(1 < 2) == false".to_string(),
                expected: object::Object::Boolean(false),
            },
            VmTest {
                input: "(1 > 2) == true".to_string(),
                expected: object::Object::Boolean(false),
            },
        ];

        run_vm_tests(tests);
    }
}
