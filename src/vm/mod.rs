pub mod error;
use error::VmError;
use crate::{code::{self, Instructions}, compiler, evaluator::object};


const STACK_SIZE: usize = 2048;

pub struct VM {
    pub constants: Vec<object::Object>,
    pub instructions: code::Instructions,
    pub stack: Vec<object::Object>,
    pub sp: usize,
}

impl VM {
    pub fn new(bytecode: compiler::Bytecode) -> Self {
        return VM {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    pub fn stack_top(&self) -> Option<&object::Object> {
        if self.sp == 0 {
            return None
        }
        Some(&self.stack[self.sp - 1])
    }

    pub fn run(&self) -> Result<(), VmError> {
        let mut ip = 0;
        while ip < self.instructions.len() {
            let opcode = instruction.into();

            match opcode {
                opcode::Constant => {
                    constant_index = code::read_u16(Instructions(self.instructions.slice(ip+1, instructions.len()));
                    ip = ip + 2;

                }
            }


        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::Compiler;

    struct VmTest {
        input: String,
        expected: object::Object,
    }

    fn parse(input: &str) -> ast::Program {
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        parser.parse_program()
    }

    fn run_vm_test(tests: Vec<VmTest>) {
        for test in tests {
            let program = parse(&test.input);
            let mut comp = Compiler::new();
            comp.compile(program).unwrap();

            vm = Vm::new(comp.bytecode());
            vm.run().unwrap();

            top_of_stack = vm.stack_top();
            test_expected_object(test.expected, top_of_stack);
        }

    }

    fn validate_integer_object(obj: object::Object, expected: usize) {
        match obj {
            object::Object::Integer(value) => assert_eq!(value, expected),
            _ => panic!("object not integer"),
        }
    }

    fn test_expected_object<T>(expected: T, actual: object::Object) {
        match expected {
            object::Object::Integer(expected) => match actual {
                object::Object::Integer(actual) => assert_eq!(expected, actual),
                _ => panic!("object not integer"),
            },
            _ => panic!("object not integer"),
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


        ]};
    }
}
