use anyhow::Result;
use interpreter::lexer::Lexer;
use interpreter::token::Token;

fn main() -> Result<()> {
    std::io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            let mut lexer = Lexer::new(line);
            while let Ok(token) = lexer.next_token() {
                println!("{} ", token);
                if let Token::Eof = token {
                    break;
                }
            }
        }
    });
    Ok(())
}
