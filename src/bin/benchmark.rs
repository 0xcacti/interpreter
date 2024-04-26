use monkey::compiler::symbol_table::SymbolTable;
use monkey::monkey::{interpret_direct, interpret_vm};
use monkey::object::builtin::Builtin;
use monkey::object::environment::Environment;
use monkey::object::Object;
use monkey::vm::GLOBAL_SIZE;

use std::{cell::RefCell, rc::Rc};

use std::time::Instant; // Adjust import paths based on your module structure

fn main() -> anyhow::Result<()> {
    let contents_direct = r#"
        let fibonacci = fn(x) {
            if (x == 0) {
                0
            } else {
                if (x == 1) {
                    return 1;
                } else {
                    fibonacci(x - 1) + fibonacci(x - 2);
                }
            }
        };
        fibonacci(35);
        "#
    .to_string();

    let contents_vm = contents_direct.clone();

    println!("Setting up direct execution...");
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));

    // Time the execution for direct interpretation
    let start = Instant::now();
    interpret_direct(
        contents_direct,
        Some(Rc::clone(&env)),
        Some(Rc::clone(&macro_env)),
    )?;
    let duration = start.elapsed();
    println!("Direct execution time: {:?}", duration);

    println!("Setting up VM execution...");
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    let constants = Rc::new(RefCell::new(vec![]));
    let symbol_table = SymbolTable::new();
    for (i, v) in Builtin::variants().iter().enumerate() {
        symbol_table.borrow_mut().define_builtin(i, v.to_string());
    }
    let globals = Rc::new(RefCell::new(vec![Rc::new(Object::Null); GLOBAL_SIZE]));
    // Time the execution for VM
    let start = Instant::now();
    interpret_vm(
        contents_vm,
        Some(Rc::clone(&macro_env)),
        symbol_table.clone(),
        constants.clone(),
        globals.clone(),
    )?;
    let duration = start.elapsed();
    println!("VM execution time: {:?}", duration);

    Ok(())
}
