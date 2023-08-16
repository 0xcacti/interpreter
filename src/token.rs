use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Illegal,
    Eof,

    Ident(String),

    // literals
    Int(String),
    String(String),
    Bool(bool),

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

    // keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return match self {
            Token::Illegal => write!(f, "Illegal"),
            Token::Eof => write!(f, "Eof"),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::Int(s) => write!(f, "Int({})", s),
            Token::Assign => write!(f, "Assign"),
            Token::Plus => write!(f, "Plus"),
            Token::Dash => write!(f, "Dash"),
            Token::Bang => write!(f, "Bang"),
            Token::Asterisk => write!(f, "Asterisk"),
            Token::Slash => write!(f, "Slash"),
            Token::Lt => write!(f, "LessThan"),
            Token::Gt => write!(f, "GreaterThan"),
            Token::Eq => write!(f, "Equal"),
            Token::NotEq => write!(f, "NotEqual"),
            Token::Comma => write!(f, "Comma"),
            Token::Semicolon => write!(f, "Semicolon"),
            Token::Lparen => write!(f, "Lparen"),
            Token::Rparen => write!(f, "Rparen"),
            Token::Lbrace => write!(f, "Lbrace"),
            Token::Rbrace => write!(f, "Rbrace"),
            Token::LBracket => write!(f, "LBracket"),
            Token::RBracket => write!(f, "RBracket"),
            Token::Function => write!(f, "Function"),
            Token::Let => write!(f, "Let"),
            Token::True => write!(f, "True"),
            Token::False => write!(f, "False"),
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::Return => write!(f, "Return"),
            Token::String(s) => write!(f, "String({})", s),
            Token::Bool(b) => write!(f, "Bool({})", b),
        };
    }
}