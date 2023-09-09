use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Illegal,
    Eof,

    Ident(String),
    Int(i64),
    String(String),

    // operators
    Assign,
    Plus,
    Dash,
    Bang,
    Asterisk,
    Slash,

    // comparators
    Lt,
    Gt,
    Eq,
    NotEq,

    Comma,
    Semicolon,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    LBracket,
    RBracket,
    Colon,

    // keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    Macro,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return match self {
            Token::Illegal => write!(f, "Illegal"),
            Token::Colon => write!(f, ":"),
            Token::Eof => write!(f, "Eof"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Int(s) => write!(f, "{}", s),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Dash => write!(f, "-"),
            Token::Bang => write!(f, "!"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Lparen => write!(f, "("),
            Token::Rparen => write!(f, ")"),
            Token::Lbrace => write!(f, "{{"),
            Token::Rbrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Function => write!(f, "fn"),
            Token::Macro => write!(f, "macro"),
            Token::Let => write!(f, "let"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),
            Token::String(s) => write!(f, "{}", s),
        };
    }
}
