use crate::token::Token;

pub enum Precedence {
    Lowest,
    Equals, // == 
    LessGreater, // > or < 
    Sum,    // +
    Product, // *
    Prefix, // -X or !X
    Call, // myFunction(X)
}
