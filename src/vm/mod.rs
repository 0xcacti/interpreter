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
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        };
    }

    pub fn stack_top(&self) -> Option<&object::Object> {
        if self.sp == 0 {
            return None;
        }
        Some(&self.stack[self.sp - 1])
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        let mut ip = 0;
        let instructions_len = self.instructions.len();

        while ip < instructions_len {
            let opcode = self.instructions[ip];

            match opcode.into() {
                Opcode::Constant => {
                    let constant_index = code::read_u16(Instructions(
                        self.instructions[ip + 1..self.instructions.len()].to_vec(),
                    ));

                    ip = ip + 2;
                    if constant_index < self.constants.len() {
                        let constant = Rc::clone(&self.constants[constant_index]);
                        self.push(constant)?;
                    } else {
                        return Err(VmError::new("Invalid constant index"));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn push(&mut self, obj: Rc<Object>) -> Result<(), VmError> {
        if self.sp >= STACK_SIZE {
            return Err(VmError::new("Stack overflow"));
        }
        self.stack[self.sp] = obj;
        self.sp += 1;
        Ok(())
    }
}

#[cfg(test)]
mod test {
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

            let top_of_stack = vm.stack_top().unwrap().clone();
            test_expected_object(test.expected, top_of_stack);
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
            VmTest {
                input: "1".to_string(),
                expected: object::Object::Integer(1),
            },
            VmTest {
                input: "2".to_string(),
                expected: object::Object::Integer(2),
            },
            VmTest {
                input: "1 + 2".to_string(),
                expected: object::Object::Integer(2),
            },
        ];
        run_vm_tests(tests);
    }
}
