use crate::token::Token;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Lexer {
    position: usize,
    read_position: usize,
    ch: u8,
    input: Vec<u8>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut lex = Lexer {
            position: 0,
            read_position: 0,
            ch: 0,
            input: input.into_bytes(),
        };
        lex.read_char();
        return lex;
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let tok = match self.ch {
            b'=' => {
                if self.peek() == b'=' {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            b'!' => {
                if self.peek() == b'=' {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'-' => Token::Dash,
            b'{' => Token::Lbrace,
            b'}' => Token::Rbrace,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident();
                return Ok(match ident.as_str() {
                    "fn" => Token::Function,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "return" => Token::Return,
                    "false" => Token::False,
                    "true" => Token::True,
                    _ => Token::Ident(ident),
                });
            }
            b'0'..=b'9' => return Ok(Token::Int(self.read_int())),
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            b'*' => Token::Asterisk,
            b'/' => Token::Slash,

            0 => Token::Eof,
            _ => Token::Illegal,
        };

        self.read_char();
        return Ok(tok);
    }

    fn read_ident(&mut self) -> String {
        let position = self.position;
        while self.ch.is_ascii_alphanumeric() || self.ch == b'_' {
            self.read_char();
        }
        return String::from_utf8_lossy(&self.input[position..self.position]).to_string();
    }

    fn read_int(&mut self) -> String {
        let position = self.position;
        while self.ch.is_ascii_alphanumeric() {
            self.read_char();
        }
        return String::from_utf8_lossy(&self.input[position..self.position]).to_string();
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn peek(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        } else {
            return self.input[self.read_position];
        }
    }
}

#[cfg(test)]
mod test {
    use super::Lexer;
    use crate::token::Token;
    use anyhow::Result;

    #[test]
    fn get_next_token() -> Result<()> {
        let input = "=+(){},;";

        let mut lexer = Lexer::new(input.into());

        let tokens = vec![
            Token::Assign,
            Token::Plus,
            Token::Lparen,
            Token::Rparen,
            Token::Lbrace,
            Token::Rbrace,
            Token::Comma,
            Token::Semicolon,
        ];

        for token in tokens {
            let next_token = lexer.next_token()?;
            println!("expected: {:?}, got: {:?}", token, next_token);
            assert_eq!(token, next_token);
        }

        return Ok(());
    }

    #[test]
    fn get_next_full_code_block() -> Result<()> {
        let input = r#"let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            };
            let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;
        if (5 < 10) {
            return true;
        } else {
            return false;
        }

        10 == 10;
        10 != 9;
        "#;
        let mut lexer = Lexer::new(input.into());

        let tokens = vec![
            Token::Let,
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Int(String::from("5")),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("ten")),
            Token::Assign,
            Token::Int(String::from("10")),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("add")),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident(String::from("x")),
            Token::Comma,
            Token::Ident(String::from("y")),
            Token::Rparen,
            Token::Lbrace,
            Token::Ident(String::from("x")),
            Token::Plus,
            Token::Ident(String::from("y")),
            Token::Semicolon,
            Token::Rbrace,
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("result")),
            Token::Assign,
            Token::Ident(String::from("add")),
            Token::Lparen,
            Token::Ident(String::from("five")),
            Token::Comma,
            Token::Ident(String::from("ten")),
            Token::Rparen,
            Token::Semicolon,
            Token::Bang,
            Token::Dash,
            Token::Slash,
            Token::Asterisk,
            Token::Int(String::from("5")),
            Token::Semicolon,
            Token::Int(String::from("5")),
            Token::Lt,
            Token::Int(String::from("10")),
            Token::Gt,
            Token::Int(String::from("5")),
            Token::Semicolon,
            Token::If,
            Token::Lparen,
            Token::Int(String::from("5")),
            Token::Lt,
            Token::Int(String::from("10")),
            Token::Rparen,
            Token::Lbrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::Rbrace,
            Token::Else,
            Token::Lbrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::Rbrace,
            Token::Int(String::from("10")),
            Token::Eq,
            Token::Int(String::from("10")),
            Token::Semicolon,
            Token::Int(String::from("10")),
            Token::NotEq,
            Token::Int(String::from("9")),
            Token::Semicolon,
            Token::Eof,
        ];

        for token in tokens {
            let next_token = lexer.next_token()?;
            println!("expected: {:?}, got: {:?}", token, next_token);
            assert_eq!(token, next_token);
        }

        return Ok(());
    }
}
