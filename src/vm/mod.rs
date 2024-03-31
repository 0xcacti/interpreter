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

                Opcode::Add => {
                    let right = self.pop();
                    let left = self.pop();

                    match (&*left, &*right) {
                        (Object::Integer(left), Object::Integer(right)) => {
                            let result = left + right;
                            self.push(Rc::new(Object::Integer(result)));
                        }
                        _ => {
                            return Err(VmError::new("Unsupported types for addition".to_string()));
                        }
                    }
                }

                Opcode::Pop => {
                    self.pop();
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
            object::Object::Integer(value) => assert_eq!(value, expected),
            _ => panic!("object not integer"),
        }
    }

    fn test_expected_object(expected: object::Object, actual: object::Object) {
        match expected {
            object::Object::Integer(expected) => validate_integer_object(actual, expected),
            _ => panic!("branch not covered"),
        }
    }

    #[test]
    fn it_adds_two_integers() {
        let tests = vec![
            // VmTest {
            //     input: "1".to_string(),
            //     expected: object::Object::Integer(1),
            // },
            // VmTest {
            //     input: "2".to_string(),
            //     expected: object::Object::Integer(2),
            // },
            VmTest {
                input: "1 + 2".to_string(),
                expected: object::Object::Integer(3),
            },
        ];
        run_vm_tests(tests);
    }
}
