use crate::token::Token;

pub enum Expression {
    Identifier(String),
}

pub enum Statement {
    Let(String, Expression),
    Return(Expression),
    Expression(Expression),
}

pub enum Node {
    Program(Vec<Statement>),
    Statement(Statement),
    Expression(Expression),
}
