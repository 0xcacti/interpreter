use anyhow::Result;
use strum_macros::{Display, EnumString};

use crate::compiler::Compiler;
use crate::evaluator::environment::Environment;
use crate::evaluator::{define_macros, evaluate, expand_macros};
use crate::vm::VM;
use crate::{utils, vm};

use crate::lexer::Lexer;
use crate::parser::ast::{self, Node};
use crate::parser::Parser;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

#[derive(Debug, Clone, EnumString, Display)]
pub enum ExecMode {
    #[strum(serialize = "vm")]
    VM,
    #[strum(serialize = "raw")]
    Raw,
}

const PROMPT: &str = ">> ";

pub fn repl(path: Option<String>, mode: ExecMode) -> Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    println!("Welcome to the Mokey Programming Language REPL!",);
    if let Some(path) = path {
        let contents = utils::load_monkey(path)?;
        interpret_chunk(
            mode.clone(),
            contents,
            Some(Rc::clone(&env)),
            Some(Rc::clone(&macro_env)),
        )?;
    }

    loop {
        print!("{}", PROMPT);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.trim() == "exit" {
            std::process::exit(0);
        }

        let chunk = interpret_chunk(
            mode.clone(),
            line,
            Some(Rc::clone(&env)),
            Some(Rc::clone(&macro_env)),
        );

        if let Err(err) = chunk {
            eprintln!("{}", err);
        }
    }
}

pub fn interpret_chunk(
    mode: ExecMode,
    contents: String,
    env: Option<Rc<RefCell<Environment>>>,
    macro_env: Option<Rc<RefCell<Environment>>>,
) -> Result<()> {
    match mode {
        ExecMode::VM => interpret_vm(contents, env, macro_env),
        ExecMode::Raw => interpret_raw(contents, env, macro_env),
    }
}

pub fn interpret_raw(
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
        evaluate(expanded, Rc::clone(&env))?;
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
    _env: Option<Rc<RefCell<Environment>>>,
    macro_env: Option<Rc<RefCell<Environment>>>,
) -> Result<()> {
    // let env = env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));
    let macro_env = macro_env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();

    match program {
        Ok(program) => {
            define_macros(&mut program.clone(), Rc::clone(&macro_env));
            let expanded = expand_macros(Node::Program(program), Rc::clone(&macro_env)).unwrap();
            let mut compiler = Compiler::new();
            compiler.compile(expanded)?;
            let mut machine = VM::new(compiler.bytecode());
            machine.run()?;
            let top = machine.stack_top();
            match top {
                Some(top) => {
                    println!("{}", top);
                }
                None => {
                    println!("No value on top of the stack");
                }
            }
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
