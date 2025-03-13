#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Invoke(Box<Expression>, Vec<Expression>),
    Primary(Primary),
}

#[derive(Debug)]
pub enum BinaryOp {
    // Logical
    And,
    Or,

    // Equality
    NotEqual,
    Equal,

    // Comparison
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Term
    Add,
    Subtract,

    // Factor
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug)]
pub enum Primary {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
    Grouping(Box<Expression>),
}
