pub mod error;
pub mod frame;

use crate::{
    code::{self, Instructions, Opcode},
    compiler,
    object::{CompiledFunction, Object},
};
use error::VmError;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use self::frame::Frame;

pub const STACK_SIZE: usize = 2048;
pub const GLOBAL_SIZE: usize = 65536;
pub const MAX_FRAMES: usize = 1024;

pub struct VM {
    pub constants: Rc<RefCell<Vec<Rc<Object>>>>,
    pub stack: Vec<Rc<Object>>,
    pub sp: usize,
    pub globals: Rc<RefCell<Vec<Rc<Object>>>>,
    pub frames: Vec<Frame>,
    pub frame_index: usize,
}

impl VM {
    pub fn new(bytecode: compiler::Bytecode) -> Self {
        let main_fn = Rc::new(CompiledFunction::new(bytecode.instructions, GLOBAL_SIZE, 0));
        let main_closure = Object::Closure(main_fn, vec![]);
        let main_frame = Frame::new(Rc::new(main_closure), 0).unwrap();

        let mut frames = vec![
            Frame::new(
                Rc::new(Object::Closure(
                    Rc::new(CompiledFunction::new(
                        Instructions::new(vec![]),
                        GLOBAL_SIZE,
                        0,
                    )),
                    vec![],
                )),
                0,
            )
            .unwrap();
            MAX_FRAMES
        ];

        frames[0] = main_frame;

        return VM {
            constants: bytecode.constants,
            stack: vec![Rc::new(Object::Null); STACK_SIZE],
            sp: 0,
            globals: Rc::new(RefCell::new(vec![Rc::new(Object::Null); GLOBAL_SIZE])),
            frames,
            frame_index: 1,
        };
    }

    pub fn new_with_global_store(
        bytecode: compiler::Bytecode,
        globals: Rc<RefCell<Vec<Rc<Object>>>>,
    ) -> Self {
        let main_fn = Rc::new(Object::Closure(
            Rc::new(CompiledFunction::new(bytecode.instructions, GLOBAL_SIZE, 0)),
            vec![],
        ));
        let main_frame = Frame::new(main_fn, 0).unwrap();

        let mut frames = vec![
            Frame::new(
                Rc::new(Object::Closure(
                    Rc::new(CompiledFunction::new(
                        Instructions::new(vec![]),
                        GLOBAL_SIZE,
                        0,
                    )),
                    vec![]
                )),
                0
            )
            .unwrap();
            MAX_FRAMES
        ];

        frames[0] = main_frame;

        return VM {
            constants: bytecode.constants,
            stack: vec![Rc::new(Object::Null); STACK_SIZE],
            sp: 0,
            globals,
            frames,
            frame_index: 1,
        };
    }

    pub fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index - 1]
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames[self.frame_index] = frame;
        self.frame_index += 1;
    }

    pub fn pop_frame(&mut self) -> &mut Frame {
        self.frame_index -= 1;
        &mut self.frames[self.frame_index]
    }

    pub fn stack_top(&self) -> Option<Rc<Object>> {
        if self.sp == 0 {
            return None;
        }
        Some(Rc::clone(&self.stack[self.sp - 1]))
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        while self.current_frame().ip < (self.current_frame().instructions()?.len() - 1) as isize {
            self.current_frame().ip += 1;

            let instructions = self.current_frame().instructions()?;
            let ip: usize = self
                .current_frame()
                .ip
                .try_into()
                .map_err(|_| VmError::new("Invalid IP".to_string()))?;

            let opcode = instructions[ip];

            match opcode.into() {
                Opcode::Constant => {
                    let constant_index = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;
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
                    let position = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip = (position - 1) as isize;
                }

                Opcode::JumpNotTruthy => {
                    let maybe_jump_position = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;
                    let condition = self.pop();
                    if !self.is_truthy(condition) {
                        self.current_frame().ip = (maybe_jump_position - 1) as isize;
                    }
                }

                Opcode::Null => {
                    self.push(Rc::new(Object::Null));
                }

                Opcode::SetGlobal => {
                    let symbol_index = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;
                    self.globals.borrow_mut()[symbol_index] = self.pop();
                }

                Opcode::GetGlobal => {
                    let symbol_index = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;

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
                    let num_elements = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;
                    let array = self.build_array(self.sp - num_elements, self.sp);
                    self.sp = self.sp - num_elements;
                    self.push(Rc::new(array));
                }

                Opcode::Hash => {
                    let num_elements = code::read_u16(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 2;
                    let hash = self.build_hash(self.sp - num_elements, self.sp);
                    self.sp = self.sp - num_elements;
                    self.push(Rc::new(hash));
                }

                Opcode::Index => {
                    let index = self.pop();
                    let indexable = self.pop();

                    self.execute_index_expression(indexable, index)?;
                }

                Opcode::Call => {
                    let num_args = code::read_u8(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 1;

                    let fun = self.stack[self.sp - 1 - num_args].clone();
                    match &*fun {
                        Object::Closure(compiled_function, num_free) => {
                            if num_args != compiled_function.num_parameters() {
                                return Err(VmError::new(format!(
                                    "Invalid number of arguments: want {}, got {}",
                                    num_args,
                                    compiled_function.num_parameters()
                                )));
                            }
                            let frame = Frame::new(fun.clone(), self.sp - num_args)?;
                            let base_pointer = frame.base_pointer;
                            self.push_frame(frame);
                            self.sp = base_pointer + compiled_function.num_locals();
                        }
                        Object::Builtin(builtin) => {
                            let args = &self.stack[self.sp - num_args..self.sp].to_vec();
                            let result = builtin
                                .apply(args)
                                .map_err(|e| VmError::new(e.to_string()))?;
                            self.sp -= num_args + 1;
                            self.push(result);
                        }
                        _ => {
                            return Err(VmError::new("Calling non-function".to_string()));
                        }
                    }
                }

                Opcode::ReturnValue => {
                    let return_value = self.pop();
                    let frame = self.pop_frame();
                    self.sp = frame.base_pointer - 1;

                    self.push(return_value);
                }

                Opcode::Return => {
                    let frame = self.pop_frame();
                    self.sp = frame.base_pointer - 1;
                    self.push(Rc::new(Object::Null));
                }

                Opcode::SetLocal => {
                    let local_index = code::read_u8(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 1;
                    let frame = self.current_frame();
                    let base_pointer = frame.base_pointer;
                    self.stack[base_pointer + local_index] = self.pop();
                }

                Opcode::GetLocal => {
                    let local_index = code::read_u8(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 1;
                    let frame = self.current_frame();
                    let base_pointer = frame.base_pointer;
                    self.push(self.stack[base_pointer + local_index].clone());
                }

                Opcode::GetBuiltin => {
                    let builtin_index = code::read_u8(&instructions, ip + 1);
                    self.current_frame().ip += 1;
                    self.push(Rc::new(Object::Builtin(builtin_index.into())));
                }

                Opcode::Closure => {
                    let const_index = code::read_u16(&instructions, ip + 1) as usize;
                    let num_free = code::read_u8(&instructions, ip + 3) as usize;
                    self.current_frame().ip += 3;
                    self.push_closure(const_index, num_free)?;
                }

                Opcode::GetFree => {
                    let free_index = code::read_u8(&instructions, ip + 1) as usize;
                    self.current_frame().ip += 1;

                    let current_closure = self.current_frame().function.clone();
                    match &*current_closure {
                        Object::Closure(_, free_vars) => {
                            self.push(free_vars[free_index].clone());
                        }
                        _ => {
                            return Err(VmError::new(
                                "tried to find free variables on non-closure".to_string(),
                            ));
                        }
                    }
                }
                Opcode::CurrentClosure => {
                    let current_closure = self.current_frame().function.clone();
                    self.push(current_closure);
                }
            }
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
        self.sp -= 1;
        let obj = self.stack[self.sp].clone();
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

    fn push_closure(&mut self, const_index: usize, num_free: usize) -> Result<(), VmError> {
        let constant = self.constants.borrow()[const_index].clone();
        match &*constant {
            Object::CompiledFunction(compiled_function) => {
                let mut free = Vec::with_capacity(num_free);
                for _ in 0..num_free {
                    free.push(self.pop());
                }
                free.reverse();

                let closure = Rc::new(Object::Closure(compiled_function.clone(), free));
                self.push(closure);
            }
            _ => {
                return Err(VmError::new("Object not compiled function".to_string()));
            }
        }
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
        expected: Result<Object, VmError>,
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
            let ret = vm.run();

            if let Err(ref ret_err) = test.expected {
                assert_eq!(ret_err.msg, test.expected.clone().unwrap_err().msg);
                return;
            }

            println!("{:?}", ret);
            assert!(ret.is_ok());

            let last = vm.last_popped_stack_elem();

            test_expected_object(test.expected.unwrap(), last.clone().deref().clone());
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
                _ => {
                    println!("{:?}", actual);
                    panic!("object not null");
                }
            },
            _ => panic!("unsupported object type"),
        }
    }

    #[test]
    fn it_adds_two_integers() {
        let tests = vec![VmTest {
            input: "1 + 2".to_string(),
            expected: Ok(Object::Integer(3)),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_subtracts_two_integers() {
        let tests = vec![VmTest {
            input: "2 - 1".to_string(),
            expected: Ok(Object::Integer(1)),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_multiplies_two_integers() {
        let tests = vec![VmTest {
            input: "2 * 2".to_string(),
            expected: Ok(Object::Integer(4)),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_divides_two_integers() {
        let tests = vec![VmTest {
            input: "4 / 2".to_string(),
            expected: Ok(Object::Integer(2)),
        }];
        run_vm_tests(tests);
    }

    #[test]
    fn it_pushes_bools() {
        let tests = vec![
            VmTest {
                input: "true".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "false".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
        ];
        run_vm_tests(tests);
    }
    #[test]
    fn it_compares() {
        let tests = vec![
            VmTest {
                input: "1 < 2".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "1 > 2".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "1 < 1".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "1 > 1".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "1 == 1".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "1 != 1".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "1 == 2".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "1 != 2".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "true == true".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "false == false".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "true == false".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "true != false".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "false != true".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "(1 < 2) == true".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "(1 < 2) == false".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "(1 > 2) == true".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_boolean_prefix_expressions() {
        let tests = vec![
            VmTest {
                input: "!true".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "!false".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "!!true".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "!!false".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "!5".to_string(),
                expected: Ok(Object::Boolean(false)),
            },
            VmTest {
                input: "!!5".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
            VmTest {
                input: "!(if (false) { 5;} )".to_string(),
                expected: Ok(Object::Boolean(true)),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_integer_prefix_expresssions() {
        let tests = vec![
            VmTest {
                input: "-5".to_string(),
                expected: Ok(Object::Integer(-5)),
            },
            VmTest {
                input: "-10".to_string(),
                expected: Ok(Object::Integer(-10)),
            },
            VmTest {
                input: "-50 + 100 + -50".to_string(),
                expected: Ok(Object::Integer(0)),
            },
            VmTest {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10".to_string(),
                expected: Ok(Object::Integer(50)),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_conditionals() {
        let tests = vec![
            VmTest {
                input: "if (true) { 10 }".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "if (true) { 10 } else { 20 }".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "if (false) { 10 } else { 20 }".to_string(),
                expected: Ok(Object::Integer(20)),
            },
            VmTest {
                input: "if (1) { 10 }".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "if (1 < 2) { 10 }".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "if (1 < 2) { 10 } else { 20 }".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "if (1 > 2) { 10 } else { 20 }".to_string(),
                expected: Ok(Object::Integer(20)),
            },
        ];
        run_vm_tests(tests);

        // test false conditionals without alternatives

        let tests = vec![
            VmTest {
                input: "if (false) { 10 }".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "if (1 > 2) { 10 }".to_string(),
                expected: Ok(Object::Null),
            },
        ];

        run_vm_tests(tests);

        // test conditionals of conditionals
        let tests = vec![VmTest {
            input: "if ((if (false) { 10 })) { 10 } else { 20 }".to_string(),
            expected: Ok(Object::Integer(20)),
        }];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_global_lets_and_gets() {
        let tests = vec![
            VmTest {
                input: "let one = 1; one".to_string(),
                expected: Ok(Object::Integer(1)),
            },
            VmTest {
                input: "let one = 1; let two = 2; one + two".to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: "let one = 1; let two = one + one; one + two".to_string(),
                expected: Ok(Object::Integer(3)),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_string_expressions() {
        let tests = vec![
            VmTest {
                input: r#""monkey""#.to_string(),
                expected: Ok(Object::String("monkey".to_string())),
            },
            VmTest {
                input: "\"mon\" + \"key\"".to_string(),
                expected: Ok(Object::String("monkey".to_string())),
            },
            VmTest {
                input: "\"mon\" + \"key\" + \"banana\"".to_string(),
                expected: Ok(Object::String("monkeybanana".to_string())),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_array_expressions() {
        let tests = vec![
            VmTest {
                input: "[]".to_string(),
                expected: Ok(Object::Array(vec![])),
            },
            VmTest {
                input: "[1, 2, 3]".to_string(),
                expected: Ok(Object::Array(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ])),
            },
            VmTest {
                input: "[1 + 2, 3 * 4, 5 + 6]".to_string(),
                expected: Ok(Object::Array(vec![
                    Rc::new(Object::Integer(3)),
                    Rc::new(Object::Integer(12)),
                    Rc::new(Object::Integer(11)),
                ])),
            },
            VmTest {
                input: r#"["a", "b", "c"] + ["e", "f", "g"]"#.to_string(),
                expected: Ok(Object::Array(vec![
                    Rc::new(Object::String("a".to_string())),
                    Rc::new(Object::String("b".to_string())),
                    Rc::new(Object::String("c".to_string())),
                    Rc::new(Object::String("e".to_string())),
                    Rc::new(Object::String("f".to_string())),
                    Rc::new(Object::String("g".to_string())),
                ])),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_hash_expressions() {
        let tests = vec![
            VmTest {
                input: "{}".to_string(),
                expected: Ok(Object::Hash(HashMap::new())),
            },
            VmTest {
                input: "{1: 2, 2: 3}".to_string(),
                expected: {
                    let mut expected_hashmap = HashMap::new();
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2)));
                    expected_hashmap
                        .insert(Rc::new(Object::Integer(2)), Rc::new(Object::Integer(3)));
                    Ok(Object::Hash(expected_hashmap))
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
                    Ok(Object::Hash(expected_hashmap))
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
                    Ok(Object::Hash(expected_hashmap))
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
                expected: Ok(Object::Integer(2)),
            },
            VmTest {
                input: "[1, 2, 3][0 + 2]".to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: "[[1, 2, 3]][0][0]".to_string(),
                expected: Ok(Object::Integer(1)),
            },
            VmTest {
                input: "[][0]".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "[1, 2, 3][99]".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "[1, 2, 3][-1]".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "{1: 1, 2: 2}[1]".to_string(),
                expected: Ok(Object::Integer(1)),
            },
            VmTest {
                input: "{1: 1, 2: 2}[2]".to_string(),
                expected: Ok(Object::Integer(2)),
            },
            VmTest {
                input: "{1: 1}[0]".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "{}[0]".to_string(),
                expected: Ok(Object::Null),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_function_calls_without_arguments() {
        let tests = vec![
            VmTest {
                input: "let fivePlusTen = fn() { 5 + 10; }; fivePlusTen();".to_string(),
                expected: Ok(Object::Integer(15)),
            },
            VmTest {
                input: "let one = fn() { 1; }; let two = fn() { 2; }; one() + two();".to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: "let a = fn() { 1; }; let b = fn() { a() + 1; }; let c = fn() { b() + 1; }; c();".to_string(),
                expected: Ok(Object::Integer(3)),
            },

            // With explicit return statement
            VmTest {
                input: "let earlyExit = fn() { return 99; 100; }; earlyExit();".to_string(),
                expected: Ok(Object::Integer(99)),
            },
            VmTest {
                input: "let earlyExit = fn() { return 99; return 100; }; earlyExit();".to_string(),
                expected: Ok(Object::Integer(99)),
            }
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_functions_without_return_value() {
        let tests = vec![
            VmTest{
                input: "let noReturn = fn() { }; noReturn();".to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: "let noReturn = fn() { }; let noReturnTwo = fn() { noReturn(); }; noReturn(); noReturnTwo();".to_string(),
                expected: Ok(Object::Null),
            }
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_functions_with_bindings() {
        let tests = vec![
            VmTest {
                input: "let one = fn() { let one = 1; one }; one();".to_string(),
                expected: Ok(Object::Integer(1)),
            },
            VmTest {
                input:
                    "let oneAndTwo = fn() { let one = 1; let two = 2; one + two; }; oneAndTwo();"
                        .to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: r#"let oneAndTwo = fn() { let one = 1; let two = 2; one + two; }; 
                    let threeAndFour = fn() { let three = 3; let four = 4; three + four; }; 
                    oneAndTwo() + threeAndFour();"#
                    .to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: r#"let firstFoobar = fn() { let foobar = 50; foobar; }; 
                    let secondFoobar = fn() { let foobar = 100; foobar; }; 
                    firstFoobar() + secondFoobar();"#
                    .to_string(),
                expected: Ok(Object::Integer(150)),
            },
            VmTest {
                input: r#"let globalSeed = 50; 
                    let minusOne = fn() { let num = 1; globalSeed - num; }; 
                    let minusTwo = fn() { let num = 2; globalSeed - num; }; 
                    minusOne() + minusTwo();"#
                    .to_string(),
                expected: Ok(Object::Integer(97)),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_functions_with_arguments_and_bindings() {
        let test = vec![
            VmTest {
                input: "let identity = fn(a) { a; }; identity(4);".to_string(),
                expected: Ok(Object::Integer(4)),
            },
            VmTest {
                input: "let sum = fn(a, b) { a + b; }; sum(1, 2);".to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: "let sum = fn(a, b) { let c = a + b; c; }; sum(1, 2);".to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: "let sum = fn(a, b) { let c = a + b; c; }; sum(1, 2) + sum(3, 4);".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: "let sum = fn(a, b) { let c = a + b; c; }; let outer = fn() { sum(1, 2) + sum(3, 4); }; outer();".to_string(),
                expected: Ok(Object::Integer(10)),
            },
            VmTest {
                input: r#"let globalNum = 10; 
                    let sum = fn(a, b) { 
                        let c = a + b; 
                        c + globalNum; 
                    }; 
                    let outer = fn() { 
                        sum(1, 2) + sum(3, 4) + globalNum; 
                    }; 
                    outer() + globalNum;"#.to_string(),
                expected: Ok(Object::Integer(50)),
            },
        ];
        run_vm_tests(test);
    }

    #[test]
    fn it_executes_calling_functions_with_wrong_arguments() {
        let tests = vec![
            VmTest {
                input: "fn() { 1; }(1);".to_string(),
                expected: Err(VmError::new(
                    "Invalid number of arguments: want 0, got 1".to_string(),
                )),
            },
            VmTest {
                input: "fn(a) { a; }();".to_string(),
                expected: Err(VmError::new(
                    "Invalid number of arguments: want 1, got 0".to_string(),
                )),
            },
            VmTest {
                input: "fn(a, b) { a + b; }(1);".to_string(),
                expected: Err(VmError::new(
                    "Invalid number of arguments: want 2, got 1".to_string(),
                )),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_builtins() {
        let tests = vec![
            VmTest {
                input: r#"len("")"#.to_string(),
                expected: Ok(Object::Integer(0)),
            },
            VmTest {
                input: r#"len("four")"#.to_string(),
                expected: Ok(Object::Integer(4)),
            },
            VmTest {
                input: r#"len("hello world")"#.to_string(),
                expected: Ok(Object::Integer(11)),
            },
            VmTest {
                input: r#"len(1)"#.to_string(),
                expected: Err(VmError::new(
                    "Argument to `len` not supported, got Integer".to_string(),
                )),
            },
            VmTest {
                input: r#"len("one", "two")"#.to_string(),
                expected: Err(VmError::new(
                    "Wrong number of arguments. got=2, want=1".to_string(),
                )),
            },
            VmTest {
                input: r#"len([1, 2, 3])"#.to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: r#"len([])"#.to_string(),
                expected: Ok(Object::Integer(0)),
            },
            VmTest {
                input: r#"echo("hello", "world")"#.to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: r#"echoln("hello", "world")"#.to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: r#"first([1, 2, 3])"#.to_string(),
                expected: Ok(Object::Integer(1)),
            },
            VmTest {
                input: r#"first([])"#.to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: r#"last([1, 2, 3])"#.to_string(),
                expected: Ok(Object::Integer(3)),
            },
            VmTest {
                input: r#"last([])"#.to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: r#"last(1)"#.to_string(),
                expected: Err(VmError::new(
                    "Argument to `last` must be ARRAY, got Integer".to_string(),
                )),
            },
            VmTest {
                input: r#"rest([1, 2, 3])"#.to_string(),
                expected: Ok(Object::Array(vec![
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ])),
            },
            VmTest {
                input: r#"rest([])"#.to_string(),
                expected: Ok(Object::Null),
            },
            VmTest {
                input: r#"push([1, 2, 3], 4)"#.to_string(),
                expected: Ok(Object::Array(vec![
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                    Rc::new(Object::Integer(4)),
                ])),
            },
            VmTest {
                input: r#"push(1, 2)"#.to_string(),
                expected: Err(VmError::new(
                    "Argument to `push` must be ARRAY, got Integer".to_string(),
                )),
            },
        ];
        run_vm_tests(tests)
    }

    #[test]
    fn it_executes_closures() {
        let tests = vec![
            VmTest {
                input: r#"
                let newClosure = fn(a) { 
                    fn() { a; }; 
                }; 
                let closure = newClosure(99); 
                closure();
                "#
                .to_string(),
                expected: Ok(Object::Integer(99)),
            },
            VmTest {
                input: r#"
                let newAdder = fn(a, b) { 
                    fn(c) { a + b + c }; 
                }; 
                let adder = newAdder(1, 2); 
                adder(8);
                "#
                .to_string(),
                expected: Ok(Object::Integer(11)),
            },
            VmTest {
                input: r#"
                let newAdder = fn(a, b) { 
                    let c = a + b; 
                    fn(d) { c + d }; 
                }; 
                let adder = newAdder(1, 2); 
                adder(8);
                "#
                .to_string(),
                expected: Ok(Object::Integer(11)),
            },
            VmTest {
                input: r#"
                let newAdderOuter = fn(a, b) { 
                    let c = a + b; 
                    fn(d) { 
                        let e = d + c; 
                        fn(f) { e + f }; 
                    }; 
                };
                let newAdderInner = newAdderOuter(1, 2);
                let adder = newAdderInner(3);
                adder(8);
                "#
                .to_string(),
                expected: Ok(Object::Integer(14)),
            },
            VmTest {
                input: r#"
                let a = 1;
                let newAdderOuter = fn(b) { 
                    fn(c) { 
                        fn(d) { a + b + c + d }; 
                    }; 
                };
                let newAdderInner = newAdderOuter(2);
                let adder = newAdderInner(3);
                adder(8);
                "#
                .to_string(),
                expected: Ok(Object::Integer(14)),
            },
            VmTest {
                input: r#"
                    let newClosure = fn(a, b) { 
                        let one = fn() { a; }; 
                        let two = fn() { b; }; 
                        fn() { one() + two(); }; 
                    };
                let closure = newClosure(9, 90);
                closure();
                "#
                .to_string(),
                expected: Ok(Object::Integer(99)),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_recursive_closures() {
        let tests = vec![
            VmTest {
                input: r#"
                let countDown = fn(x) {
                    if (x == 0) {
                        return 0;
                    } else {
                        countDown(x - 1);
                    }
                };
                countDown(1);
                "#
                .to_string(),
                expected: Ok(Object::Integer(0)),
            },
            VmTest {
                input: r#"
                let countDown = fn(x) {
                    if (x == 0) {
                        return 0;
                    } else {
                        countDown(x - 1);
                    }
                };
                let wrapper = fn() {
                    countDown(1);
                };
                wrapper();
                "#
                .to_string(),
                expected: Ok(Object::Integer(0)),
            },
            VmTest {
                input: r#"
                let wrapper = fn() {
                    let countDown = fn(x) {
                        if (x == 0) {
                            return 0;
                        } else {
                            countDown(x - 1);
                        }
                    };
                    countDown(1);
                };
                wrapper();
                "#
                .to_string(),
                expected: Ok(Object::Integer(0)),
            },
        ];
        run_vm_tests(tests);
    }

    #[test]
    fn it_executes_recursive_fibonacci() {
        let tests = vec![VmTest {
            input: r#"
                let fibonacci = fn(x) {
                    if (x == 0) {
                        return 0;
                    } else {
                        if (x == 1) {
                            return 1;
                        } else {
                            fibonacci(x - 1) + fibonacci(x - 2);
                        }
                    }
                };
                fibonacci(15);
                "#
            .to_string(),
            expected: Ok(Object::Integer(610)),
        }];
        run_vm_tests(tests);
    }
}
