use crate::token::Token;

pub struct Lexer {
    position: usize,
    read_position: usize,
    ch: u8,
    input: Vec<u8>,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut lex = Lexer {
            position: 0,
            read_position: 0,
            ch: 0,
            input: input.as_bytes().to_vec(),
        };
        lex.read_char();
        return lex;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.ch == b'/' {
            match self.peek() {
                b'/' => {
                    self.skip_line_comment();
                    return self.next_token();
                }
                b'*' => {
                    self.skip_block_comment();
                    return self.next_token();
                }
                _ => {}
            }
        }

        let tok = match self.ch {
            b'=' => self.single_or_double(b'=', Token::Assign, Token::Eq),
            b'!' => self.single_or_double(b'=', Token::Bang, Token::NotEq),
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'-' => Token::Dash,
            b'{' => Token::Lbrace,
            b'}' => Token::Rbrace,
            b':' => Token::Colon,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident();
                return match ident.as_str() {
                    "fn" => Token::Function,
                    "macro" => Token::Macro,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "return" => Token::Return,
                    "false" => Token::False,
                    "true" => Token::True,
                    _ => Token::Ident(ident),
                };
            }
            b'0'..=b'9' => return Token::Int(self.read_int().parse::<i64>().unwrap()),
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            b'*' => Token::Asterisk,
            b'/' => Token::Slash,
            b'"' => Token::String(self.read_string()),

            0 => Token::Eof,
            _ => Token::Illegal,
        };

        self.read_char();
        return tok;
    }

    fn skip_line_comment(&mut self) {
        self.read_char();
        self.read_char();
        while self.ch != b'\n' && self.ch != 0 {
            self.read_char();
        }
        if self.ch != 0 {
            self.read_char();
        }
    }

    fn skip_block_comment(&mut self) {
        self.read_char();
        self.read_char();
        let mut nesting = 1;
        while nesting > 0 && self.ch != 0 {
            if self.ch == b'/' && self.peek() == b'*' {
                self.read_char();
                self.read_char();
                nesting += 1;
            } else if self.ch == b'*' && self.peek() == b'/' {
                self.read_char();
                self.read_char();
                nesting -= 1;
            } else {
                self.read_char();
            }
        }
    }

    fn read_string(&mut self) -> String {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == b'"' || self.ch == 0 {
                break;
            }
        }
        return String::from_utf8_lossy(&self.input[position..self.position]).to_string();
    }

    fn single_or_double(
        &mut self,
        expected_next: u8,
        single_token: Token,
        double_token: Token,
    ) -> Token {
        if self.peek() == expected_next {
            self.read_char();
            return double_token;
        }
        single_token
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
    fn it_gets_next_token_correctly() -> Result<()> {
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
            let next_token = lexer.next_token();
            println!("expected: {:?}, got: {:?}", token, next_token);
            assert_eq!(token, next_token);
        }

        return Ok(());
    }

    #[test]
    fn it_lexes_whole_code_blocks() -> Result<()> {
        let input = r#"let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            };
            let result = add(five, ten);
        !-/ *5;
        5 < 10 > 5;
        if (5 < 10) {
            return true;
        } else {
            return false;
        }

        10 == 10;
        10 != 9; // test comment
        "foobar"
        /* 
         *
         */
        "foo bar" 
        [1, 2];
        {"foo": "bar"}
        macro(x, y) { x + y; };
        "#;
        let mut lexer = Lexer::new(input.into());

        let tokens = vec![
            Token::Let,
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Int(5),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("ten")),
            Token::Assign,
            Token::Int(10),
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
            Token::Int(5),
            Token::Semicolon,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Gt,
            Token::Int(5),
            Token::Semicolon,
            Token::If,
            Token::Lparen,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
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
            Token::Int(10),
            Token::Eq,
            Token::Int(10),
            Token::Semicolon,
            Token::Int(10),
            Token::NotEq,
            Token::Int(9),
            Token::Semicolon,
            Token::String(String::from("foobar")),
            Token::String(String::from("foo bar")),
            Token::LBracket,
            Token::Int(1),
            Token::Comma,
            Token::Int(2),
            Token::RBracket,
            Token::Semicolon,
            Token::Lbrace,
            Token::String(String::from("foo")),
            Token::Colon,
            Token::String(String::from("bar")),
            Token::Rbrace,
            Token::Macro,
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
            Token::Eof,
        ];

        for token in tokens {
            let next_token = lexer.next_token();
            println!("expected: {:?}, got: {:?}", token, next_token);
            assert_eq!(token, next_token);
        }

        return Ok(());
    }
}
