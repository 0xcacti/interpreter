use anyhow::Result;
use signal_hook::{consts::SIGINT, iterator::Signals};
use strum_macros::{Display, EnumString};

use crate::compiler::symbol_table::SymbolTable;
use crate::compiler::Compiler;
use crate::evaluator::{define_macros, evaluate, expand_macros};
use crate::object::builtin::Builtin;
use crate::object::environment::Environment;
use crate::object::Object;
use crate::utils;
use crate::vm::{GLOBAL_SIZE, VM};

use crate::lexer::Lexer;
use crate::parser::ast::Node;
use crate::parser::Parser;
use std::thread;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

#[derive(Debug, Clone, EnumString, Display)]
pub enum ExecMode {
    #[strum(serialize = "vm")]
    VM,
    #[strum(serialize = "direct")]
    Direct,
}

const PROMPT: &str = ">> ";

pub fn repl(path: Option<String>, mode: ExecMode) -> Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    println!("Welcome to the Mokey Programming Language REPL!",);

    let mut signals = Signals::new(&[SIGINT])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    println!("Exiting REPL");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });

    let constants = Rc::new(RefCell::new(vec![]));
    let symbol_table = SymbolTable::new();
    for (i, v) in Builtin::variants().iter().enumerate() {
        symbol_table.borrow_mut().define_builtin(i, v.to_string());
    }
    let globals = Rc::new(RefCell::new(vec![Rc::new(Object::Null); GLOBAL_SIZE]));

    if let Some(path) = path {
        let contents = utils::load_monkey(path)?;

        let result = match mode {
            ExecMode::Direct => {
                interpret_direct(contents, Some(Rc::clone(&env)), Some(Rc::clone(&macro_env)))
            }
            ExecMode::VM => interpret_vm(
                contents,
                Some(Rc::clone(&macro_env)),
                symbol_table.clone(),
                constants.clone(),
                globals.clone(),
            ),
        };

        if let Err(err) = result {
            eprintln!("{}", err);
        }
    }

    loop {
        print!("{}", PROMPT);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.trim() == "exit" {
            std::process::exit(0);
        }

        let result = match mode {
            ExecMode::Direct => {
                interpret_direct(line, Some(Rc::clone(&env)), Some(Rc::clone(&macro_env)))
            }
            ExecMode::VM => interpret_vm(
                line,
                Some(Rc::clone(&macro_env)),
                symbol_table.clone(),
                constants.clone(),
                globals.clone(),
            ),
        };

        if let Err(err) = result {
            eprintln!("{}", err);
        }
    }
}

pub fn interpret_chunk(mode: ExecMode, contents: String) -> Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));

    let constants = Rc::new(RefCell::new(vec![]));
    let symbol_table = SymbolTable::new();
    for (i, v) in Builtin::variants().iter().enumerate() {
        symbol_table.borrow_mut().define_builtin(i, v.to_string());
    }
    let globals = Rc::new(RefCell::new(vec![Rc::new(Object::Null); GLOBAL_SIZE]));

    let result = match mode {
        ExecMode::Direct => {
            interpret_direct(contents, Some(Rc::clone(&env)), Some(Rc::clone(&macro_env)))
        }
        ExecMode::VM => interpret_vm(
            contents,
            Some(Rc::clone(&macro_env)),
            symbol_table.clone(),
            constants.clone(),
            globals.clone(),
        ),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
    }

    Ok(())
}

pub fn interpret_direct(
    contents: String,
    env: Option<Rc<RefCell<Environment>>>,
    macro_env: Option<Rc<RefCell<Environment>>>,
) -> Result<()> {
    let env = env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));
    let macro_env = macro_env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();
    if let Ok(mut program) = program {
        define_macros(&mut program, Rc::clone(&macro_env));
        let expanded =
            expand_macros(Node::Program(program.clone()), Rc::clone(&macro_env)).unwrap();
        let result = evaluate(expanded, Rc::clone(&env));
        if let Ok(result) = result {
            println!("{}", result);
        }
    } else if let Err(err) = &program {
        println!("Woops! We ran into some monkey business here!");
        println!("parser errors:");
        for e in err {
            eprintln!("\t{}", e);
        }
    }
    Ok(())
}

pub fn interpret_vm(
    contents: String,
    macro_env: Option<Rc<RefCell<Environment>>>,
    symbol_table: Rc<RefCell<SymbolTable>>,
    constants: Rc<RefCell<Vec<Rc<Object>>>>,
    globals: Rc<RefCell<Vec<Rc<Object>>>>,
) -> Result<()> {
    // let env = env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));
    let macro_env = macro_env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();

    match program {
        Ok(program) => {
            // expand macros
            define_macros(&mut program.clone(), Rc::clone(&macro_env));
            let expanded = expand_macros(Node::Program(program), Rc::clone(&macro_env)).unwrap();

            // compile
            let mut compiler = Compiler::new_with_state(symbol_table, constants);
            compiler.compile(expanded)?;

            let code = compiler.bytecode();

            let mut machine = VM::new_with_global_store(code, globals);
            machine.run()?;
            let last_elem = machine.last_popped_stack_elem();
            println!("{}", last_elem);
        }
        Err(err) => {
            println!("Woops! We ran into some monkey business here!");
            println!("parser errors:");
            for e in err {
                eprintln!("\t{}", e);
            }
        }
    }
    Ok(())
}
