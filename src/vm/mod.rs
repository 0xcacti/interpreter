pub mod error;

use crate::{
    code::{self, Opcode},
    compiler,
    evaluator::object::Object,
};
use error::VmError;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub const STACK_SIZE: usize = 2048;
pub const GLOBAL_SIZE: usize = 65536;

pub struct VM {
    pub constants: Rc<RefCell<Vec<Rc<Object>>>>,
    pub instructions: code::Instructions,
    pub stack: Vec<Rc<Object>>,
    pub sp: usize,
    pub globals: Rc<RefCell<Vec<Rc<Object>>>>,
}

impl VM {
    pub fn new(bytecode: compiler::Bytecode) -> Self {
        return VM {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: vec![Rc::new(Object::Null); STACK_SIZE],
            sp: 0,
            globals: Rc::new(RefCell::new(vec![Rc::new(Object::Null); GLOBAL_SIZE])),
        };
    }

    pub fn new_with_global_store(
        bytecode: compiler::Bytecode,
        globals: Rc<RefCell<Vec<Rc<Object>>>>,
    ) -> Self {
        return VM {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: vec![Rc::new(Object::Null); STACK_SIZE],
            sp: 0,
            globals,
        };
    }

    pub fn stack_top(&self) -> Option<Rc<Object>> {
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
                    let constants = self.constants.borrow().clone();

                    if constant_index > constants.len() {
                        return Err(VmError::new("Invalid constant index".to_string()));
                    }
                    let constant = Rc::clone(&constants[constant_index]);
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

                Opcode::Bang => {
                    self.execute_bang_operator()?;
                }

                Opcode::Minus => {
                    self.execute_minus_operator()?;
                }

                Opcode::Jump => {
                    let position = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip = position - 1;
                }

                Opcode::JumpNotTruthy => {
                    let maybe_jump_position = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip = ip + 2;
                    let condition = self.pop();
                    if !self.is_truthy(condition) {
                        ip = maybe_jump_position - 1;
                    }
                }

                Opcode::Null => {
                    self.push(Rc::new(Object::Null));
                }

                Opcode::SetGlobal => {
                    let symbol_index = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip = ip + 2;
                    self.globals.borrow_mut()[symbol_index] = self.pop();
                }

                Opcode::GetGlobal => {
                    let symbol_index = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip += 2;

                    // Clone the global variable before borrowing mutably
                    let global = self.globals.borrow().get(symbol_index).cloned();

                    // Check if the global variable exists at the given index
                    if let Some(global) = global {
                        // Push the cloned global variable onto the stack
                        self.push(global);
                    } else {
                        // Handle the case when the global variable doesn't exist
                        return Err(VmError::new("Global variable not found".to_string()));
                    }
                }

                Opcode::Array => {
                    let num_elements = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip += 2;
                    let array = self.build_array(self.sp - num_elements, self.sp);
                    self.sp = self.sp - num_elements;
                    self.push(Rc::new(array));
                }

                Opcode::Hash => {
                    let num_elements = code::read_u16(&self.instructions, ip + 1) as usize;
                    ip += 2;
                    let hash = self.build_hash(self.sp - num_elements, self.sp);
                    self.sp = self.sp - num_elements;
                    self.push(Rc::new(hash));
                }

                Opcode::Index => {
                    let index = self.pop();
                    let indexable = self.pop();

                    self.execute_index_expression(indexable, index)?;
                }
                _ => {
                    return Err(VmError::new("Invalid opcode".to_string()));
                }
            }
            ip += 1;
        }
        Ok(())
    }

    fn is_truthy(&self, obj: Rc<Object>) -> bool {
        let truthy = match *obj {
            Object::Boolean(b) => b,
            Object::Null => false,
            _ => true,
        };
        truthy
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

    fn execute_index_expression(
        &mut self,
        indexable: Rc<Object>,
        index: Rc<Object>,
    ) -> Result<(), VmError> {
        match &*indexable {
            Object::Array(arr) => match &*index {
                Object::Integer(real_index) => {
                    let max = arr.len() as i64;
                    if *real_index < 0 || *real_index >= max {
                        self.push(Rc::new(Object::Null));
                    } else {
                        self.push(arr[*real_index as usize].clone());
                    }
                    Ok(())
                }
                _ => return Err(VmError::new("Unsupported index type for array".to_string())),
            },
            Object::Hash(hash) => {
                match hash.get(&index) {
                    Some(obj) => self.push(obj.clone()),
                    None => self.push(Rc::new(Object::Null)),
                }
                Ok(())
            }

            _ => {
                return Err(VmError::new(
                    "Unsupported operation index for type".to_string(),
                ))
            }
        }
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
            (Object::String(left), Object::String(right)) => {
                let result = match opcode {
                    Opcode::Add => format!("{}{}", left, right),
                    _ => {
                        return Err(VmError::new("Unsupported operation for string".to_string()));
                    }
                };
                self.push(Rc::new(Object::String(result)));
            }
            (Object::Array(left), Object::Array(right)) => {
                let result = match opcode {
                    Opcode::Add => {
                        let mut new_array = left.clone();
                        new_array.extend(right.clone());
                        new_array
                    }
                    _ => {
                        return Err(VmError::new("Unsupported operation for array".to_string()));
                    }
                };
                self.push(Rc::new(Object::Array(result)));
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

    pub fn execute_bang_operator(&mut self) -> Result<(), VmError> {
        let operand = self.pop();
        match &*operand {
            Object::Boolean(value) => {
                let result = Rc::new(Object::Boolean(!value));
                self.push(result);
            }

            Object::Null => {
                let result = Rc::new(Object::Boolean(true));
                self.push(result);
            }
            _ => {
                let result = Rc::new(Object::Boolean(false));
                self.push(result);
            }
        }
        Ok(())
    }

    pub fn execute_minus_operator(&mut self) -> Result<(), VmError> {
        let operand = self.pop();
        match &*operand {
            Object::Integer(value) => {
                let result = Rc::new(Object::Integer(-value));
                self.push(result);
            }
            _ => {
                return Err(VmError::new("Unsupported type for negation".to_string()));
            }
        }
        Ok(())
    }

    fn build_array(&mut self, start_index: usize, end_index: usize) -> Object {
        let mut elements = vec![Rc::new(Object::Null); end_index - start_index];
        for i in start_index..end_index {
            elements[i - start_index] = self.stack[i].clone();
        }
        Object::Array(elements)
    }

    fn build_hash(&mut self, start_index: usize, end_index: usize) -> Object {
        let mut pairs = HashMap::new();
        let mut i = start_index;
        while start_index <= i && i < end_index {
            let key = self.stack[i].clone();
            let value = self.stack[i + 1].clone();
            pairs.insert(key, value);
            i += 2;
        }
        Object::Hash(pairs)
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, ops::Deref};

    use super::*;
    use crate::{
        compiler::Compiler,
        lexer::Lexer,
        parser::{ast, Parser},
    };

    struct VmTest {
        input: String,
        expected: Object,
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

    fn validate_integer_object(obj: Object, expected: i64) {
        match obj {
            Object::Integer(value) => assert_eq!(value, expected),
            _ => panic!("object not integer"),
        }
    }

    fn validate_boolean_object(obj: Object, expected: bool) {
        match obj {
            Object::Boolean(value) => assert_eq!(value, expected),
            _ => panic!("object not boolean"),
        }
    }

    fn validate_string_object(obj: Object, expected: &str) {
        match obj {
            Object::String(value) => assert_eq!(value, expected),
            _ => panic!("object not string"),
        }
    }

    fn validate_array_object(obj: Object, expected: Vec<Rc<Object>>) {
        match obj {
            Object::Array(value) => {
                for (i, v) in value.iter().enumerate() {
                    test_expected_object(
                        expected[i].clone().deref().clone(),
                        v.clone().deref().clone(),
                    );
                }
            }
            _ => panic!("object not array"),
        }
    }

    fn validate_hash_object(obj: Object, expected: HashMap<Rc<Object>, Rc<Object>>) {
        match obj {
            Object::Hash(value) => {
                // copy to comparable type
                let expected_hm = deref_hashmaps_by_copy(&expected);
                let actual_hm = deref_hashmaps_by_copy(&value);
                println!("expected: {:?}", expected_hm);
                println!("actual: {:?}", actual_hm);

                assert!(actual_hm.eq(&expected_hm));
                // for (k, v) in expected_hm.iter() {
                //     let ak = actual_hm.get_key_value(k).unwrap().0;
                //     let av = actual_hm.get_key_value(k).unwrap().1;

                //     test_expected_object(k.clone(), ak.clone());
                //     test_expected_object(v.clone(), av.clone());
                // }
            }
            _ => panic!("object not hash"),
        }
    }

    fn deref_hashmaps_by_copy(hm: &HashMap<Rc<Object>, Rc<Object>>) -> HashMap<Object, Object> {
        let mut object_hm = HashMap::new();
        for (k, v) in hm.iter() {
            object_hm.insert(k.clone().deref().clone(), v.clone().deref().clone());
        }
        object_hm
    }

    fn test_expected_object(expected: Object, actual: Object) {
        match expected {
            Object::Integer(expected) => validate_integer_object(actual, expected),
            Object::Boolean(expected) => validate_boolean_object(actual, expected),
            Object::String(expected) => validate_string_object(actual, &expected),
            Object::Array(expected) => validate_array_object(actual, expected),
            Object::Hash(expected) => validate_hash_object(actual, expected),
            Object::Null => match actual {
                Object::Null => {}
                _ => panic!("object not null"),
            },
            _ => panic!("unsupported object type"),
        }
    }

    #[test]
    fn it_adds_two_integers() {
        let tests = vec![VmTest {
            input: "1 + 2".to_string(),
            expected: Object::Integer(3),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_subtracts_two_integers() {
        let tests = vec![VmTest {
            input: "2 - 1".to_string(),
            expected: Object::Integer(1),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_multiplies_two_integers() {
        let tests = vec![VmTest {
            input: "2 * 2".to_string(),
            expected: Object::Integer(4),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_divides_two_integers() {
        let tests = vec![VmTest {
            input: "4 / 2".to_string(),
            expected: Object::Integer(2),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_pushes_bools() {
        let tests = vec![
            VmTest {
                input: "true".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "false".to_string(),
                expected: Object::Boolean(false),
            },
        ];
        run_vm_tests(tests);
    }
    #[test]
    fn it_compares() {
        let tests = vec![
            VmTest {
                input: "1 < 2".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "1 > 2".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "1 < 1".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "1 > 1".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "1 == 1".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "1 != 1".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "1 == 2".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "1 != 2".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "true == true".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "false == false".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "true == false".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "true != false".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "false != true".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "(1 < 2) == true".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "(1 < 2) == false".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "(1 > 2) == true".to_string(),
                expected: Object::Boolean(false),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_boolean_prefix_expressions() {
        let tests = vec![
            VmTest {
                input: "!true".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "!false".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "!!true".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "!!false".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "!5".to_string(),
                expected: Object::Boolean(false),
            },
            VmTest {
                input: "!!5".to_string(),
                expected: Object::Boolean(true),
            },
            VmTest {
                input: "!(if (false) { 5;} )".to_string(),
                expected: Object::Boolean(true),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_integer_prefix_expresssions() {
        let tests = vec![
            VmTest {
                input: "-5".to_string(),
                expected: Object::Integer(-5),
            },
            VmTest {
                input: "-10".to_string(),
                expected: Object::Integer(-10),
            },
            VmTest {
                input: "-50 + 100 + -50".to_string(),
                expected: Object::Integer(0),
            },
            VmTest {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10".to_string(),
                expected: Object::Integer(50),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_conditionals() {
        let tests = vec![
            VmTest {
                input: "if (true) { 10 }".to_string(),
                expected: Object::Integer(10),
            },
            VmTest {
                input: "if (true) { 10 } else { 20 }".to_string(),
                expected: Object::Integer(10),
            },
            VmTest {
                input: "if (false) { 10 } else { 20 }".to_string(),
                expected: Object::Integer(20),
            },
            VmTest {
                input: "if (1) { 10 }".to_string(),
                expected: Object::Integer(10),
            },
            VmTest {
                input: "if (1 < 2) { 10 }".to_string(),
                expected: Object::Integer(10),
            },
            VmTest {
                input: "if (1 < 2) { 10 } else { 20 }".to_string(),
                expected: Object::Integer(10),
            },
            VmTest {
                input: "if (1 > 2) { 10 } else { 20 }".to_string(),
                expected: Object::Integer(20),
            },
        ];
        run_vm_tests(tests);

        // test false conditionals without alternatives

        let tests = vec![
            VmTest {
                input: "if (false) { 10 }".to_string(),
                expected: Object::Null,
            },
            VmTest {
                input: "if (1 > 2) { 10 }".to_string(),
                expected: Object::Null,
            },
        ];

        run_vm_tests(tests);

        // test conditionals of conditionals
        let tests = vec![VmTest {
            input: "if ((if (false) { 10 })) { 10 } else { 20 }".to_string(),
            expected: Object::Integer(20),
        }];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_global_lets_and_gets() {
        let tests = vec![
            VmTest {
                input: "let one = 1; one".to_string(),
                expected: Object::Integer(1),
            },
            VmTest {
                input: "let one = 1; let two = 2; one + two".to_string(),
                expected: Object::Integer(3),
            },
            VmTest {
                input: "let one = 1; let two = one + one; one + two".to_string(),
                expected: Object::Integer(3),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_string_expressions() {
        let tests = vec![
            VmTest {
                input: r#""monkey""#.to_string(),
                expected: Object::String("monkey".to_string()),
            },
            VmTest {
                input: "\"mon\" + \"key\"".to_string(),
                expected: Object::String("monkey".to_string()),
            },
            VmTest {
                input: "\"mon\" + \"key\" + \"banana\"".to_string(),
                expected: Object::String("monkeybanana".to_string()),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_array_expressions() {
        let tests = vec![
            VmTest {
                input: "[]".to_string(),
                expected: Object::Array(vec![]),
            },
            VmTest {
                input: "[1, 2, 3]".to_string(),
                expected: Object::Array(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ]),
            },
            VmTest {
                input: "[1 + 2, 3 * 4, 5 + 6]".to_string(),
                expected: Object::Array(vec![
                    Rc::new(Object::Integer(3)),
                    Rc::new(Object::Integer(12)),
                    Rc::new(Object::Integer(11)),
                ]),
            },
            VmTest {
                input: r#"["a", "b", "c"] + ["e", "f", "g"]"#.to_string(),
                expected: Object::Array(vec![
                    Rc::new(Object::String("a".to_string())),
                    Rc::new(Object::String("b".to_string())),
                    Rc::new(Object::String("c".to_string())),
                    Rc::new(Object::String("e".to_string())),
                    Rc::new(Object::String("f".to_string())),
                    Rc::new(Object::String("g".to_string())),
                ]),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_hash_expressions() {
        let tests = vec![
            VmTest {
                input: "{}".to_string(),
                expected: Object::Hash(HashMap::new()),
            },
            VmTest {
                input: "{1: 2, 2: 3}".to_string(),
                expected: {
                    let mut expected_hashmap = HashMap::new();
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2)));
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(2)), Rc::new(Object::Integer(3)));
                    Object::Hash(expected_hashmap)
                },
            },
            VmTest {
                input: "{1+1: 2*2, 3+3: 4*4}".to_string(),
                expected: {
                    let mut expected_hashmap = HashMap::new();
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(2)), Rc::new(Object::Integer(4)));
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(6)), Rc::new(Object::Integer(16)));
                    Object::Hash(expected_hashmap)
                },
            },
            VmTest {
                input: r#"{"a": 12, "a" + "b": [1, 2, 3, "z"]}"#.to_string(),
                expected: {
                    let mut expected_hashmap = HashMap::new();
                    expected_hashmap.insert(
                        Rc::new(Object::String("a".to_string())),
                        Rc::new(Object::Integer(12)),
                    );
                    expected_hashmap.insert(
                        Rc::new(Object::String("ab".to_string())),
                        Rc::new(Object::Array(vec![
                            Rc::new(Object::Integer(1)),
                            Rc::new(Object::Integer(2)),
                            Rc::new(Object::Integer(3)),
                            Rc::new(Object::String("z".to_string())),
                        ])),
                    );
                    Object::Hash(expected_hashmap)
                },
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_index_expressions() {
        let tests = vec![
            VmTest {
                input: "[1, 2, 3][1]".to_string(),
                expected: Object::Integer(2),
            },
            VmTest {
                input: "[1, 2, 3][0 + 2]".to_string(),
                expected: Object::Integer(3),
            },
            VmTest {
                input: "[[1, 2, 3]][0][0]".to_string(),
                expected: Object::Integer(1),
            },
            VmTest {
                input: "[][0]".to_string(),
                expected: Object::Null,
            },
            VmTest {
                input: "[1, 2, 3][99]".to_string(),
                expected: Object::Null,
            },
            VmTest {
                input: "[1, 2, 3][-1]".to_string(),
                expected: Object::Null,
            },
            VmTest {
                input: "{1: 1, 2: 2}[1]".to_string(),
                expected: Object::Integer(1),
            },
            VmTest {
                input: "{1: 1, 2: 2}[2]".to_string(),
                expected: Object::Integer(2),
            },
            VmTest {
                input: "{1: 1}[0]".to_string(),
                expected: Object::Null,
            },
            VmTest {
                input: "{}[0]".to_string(),
                expected: Object::Null,
            },
        ];

        run_vm_tests(tests);
    }
}
