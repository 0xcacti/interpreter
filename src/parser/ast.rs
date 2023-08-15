pub type Program = Vec<Statement>;

pub enum Expression {
    Identifier(String),
    Literal(String),
}

pub enum Statement {
    Let(String, Expression),
    Return(Expression),
    Expression(Expression),
}

// define node enum - our parser works on statement and expression nodes
pub enum Node {
    Statement(Statement),
    Expression(Expression),
}
