use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> Token;
}

pub trait Statement {
    fn statement_node(&self);
}

pub trait Expression {
    fn expression_node(&self);
}

pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

pub struct LetStatement<'a> {
    pub token: Token,
    pub name: &'a Identifier,
    pub value: Box<dyn Expression>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

impl Node for Identifier {
    fn token_literal(&self) -> Token {
        self.token
    }
}
