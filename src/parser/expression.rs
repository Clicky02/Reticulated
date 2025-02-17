#[derive(Debug)]
pub enum Expression {
    Logical(Box<Expression>, LogicalOp, Box<Expression>),
    Equality(Box<Expression>, EqualityOp, Box<Expression>),
    Comparison(Box<Expression>, ComparisonOp, Box<Expression>),
    Term(Box<Expression>, TermOp, Box<Expression>),
    Factor(Box<Expression>, FactorOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Invoke(Box<Expression>, Vec<Expression>),
    Primary(Primary),
}

#[derive(Debug)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug)]
pub enum EqualityOp {
    NotEqual,
    Equal,
}

#[derive(Debug)]
pub enum ComparisonOp {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug)]
pub enum TermOp {
    Add,
    Subtract,
}

#[derive(Debug)]
pub enum FactorOp {
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
