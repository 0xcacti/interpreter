use crate::token::Token;

#[derive(PartialOrd, PartialEq, Debug, Copy, Clone)]
pub enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(X)
}

pub fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Eq | Token::NotEq => Precedence::Equals,
        Token::Plus | Token::Dash => Precedence::Sum,
        Token::Lt | Token::Gt => Precedence::LessGreater,
        Token::Slash | Token::Asterisk => Precedence::Product,
        Token::Lparen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}
