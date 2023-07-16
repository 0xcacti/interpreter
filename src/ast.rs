use crate::token::Token;

enum Node {
    Statement,
    Expression,
}

pub struct Program {
    // will only contain statements
    pub statements: Vec<Node>,
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
