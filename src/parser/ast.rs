pub type Program = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Identifier, Expression),
    ReturnStatement(Expression),
    ExpressionStatement(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    IdentifierExpression(Identifier),
    LiteralExpression(Literal),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Identifier {
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    StringLiteral(String),
}
