use token::Token;

pub trait Node {
    fn token_literal(&self) -> Token;
}

pub trait StatementNode {
    fn statement_node(&self);
}

pub trait ExpressionNode {
    fn expression_node(&self);
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
    pub name: &Identifier,
    pub value: Expression,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }
}

impl ExpressionNode for Identifier {
    fn expression_node(&self) {}
}

impl Node for Identifier {
    fn token_literal(&self) -> Token {
        self.token
    }
}
