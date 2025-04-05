#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Invoke(Box<Expression>, Vec<Expression>),
    Access(Box<Expression>, String),
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

impl BinaryOp {
    pub fn fn_name(&self) -> &str {
        match self {
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::NotEqual => "__ne__",
            BinaryOp::Equal => "__eq__",
            BinaryOp::Greater => "__gt__",
            BinaryOp::GreaterEqual => "__ge__",
            BinaryOp::Less => "__lt__",
            BinaryOp::LessEqual => "__le__",
            BinaryOp::Add => "__add__",
            BinaryOp::Subtract => "__sub__",
            BinaryOp::Multiply => "__mul__",
            BinaryOp::Divide => "__truediv__",
            BinaryOp::Modulo => "__mod__",
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Negate,
}

impl UnaryOp {
    pub fn fn_name(&self) -> &str {
        match self {
            UnaryOp::Not => todo!(),
            UnaryOp::Negate => "__neg__",
        }
    }
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
