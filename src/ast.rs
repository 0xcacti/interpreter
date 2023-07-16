use crate::token::Token;

enum Statement {}
enum Expression {}

enum Node {
    Statement(Statement),
    Expression(Expression),
}

pub struct Program {
    pub statements: Vec<Statement>,
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
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
