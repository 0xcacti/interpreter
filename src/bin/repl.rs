use anyhow::Result;
use interpreter::lexer::Lexer;
use interpreter::token::Token;
use users::get_current_username;

fn main() -> Result<()> {
    println!(
        "Dear {}, Welcome to the Mokey Programming Language REPL!",
        get_current_username().unwrap().to_string_lossy()
    );
    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let mut lexer = Lexer::new(line);
            loop {
                let token = lexer.next_token();
                println!("{} ", token);
                if token == Token::Eof {
                    break;
                }
            }
        }
    });
    Ok(())
}
