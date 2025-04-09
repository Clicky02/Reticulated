#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    BinaryFn(Box<Expression>, BinaryFnOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    UnaryFn(UnaryFnOp, Box<Expression>),
    Invoke(Box<Expression>, Vec<Expression>),
    Access(Box<Expression>, String),
    Primary(Primary),
}

#[derive(Debug)]
pub enum BinaryOp {
    // Logical
    And,
    Or,
}

impl BinaryOp {
    pub fn to_string(&self) -> &str {
        match self {
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        }
    }
}

#[derive(Debug)]
pub enum BinaryFnOp {
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
    Exponentiate,
    MatMul,
}

impl BinaryFnOp {
    pub fn fn_name(&self) -> &str {
        match self {
            BinaryFnOp::NotEqual => "__ne__",
            BinaryFnOp::Equal => "__eq__",
            BinaryFnOp::Greater => "__gt__",
            BinaryFnOp::GreaterEqual => "__ge__",
            BinaryFnOp::Less => "__lt__",
            BinaryFnOp::LessEqual => "__le__",
            BinaryFnOp::Add => "__add__",
            BinaryFnOp::Subtract => "__sub__",
            BinaryFnOp::Multiply => "__mul__",
            BinaryFnOp::Divide => "__truediv__",
            BinaryFnOp::Modulo => "__mod__",
            BinaryFnOp::Exponentiate => "__pow__",
            BinaryFnOp::MatMul => "__matmul__",
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
}

impl UnaryOp {
    pub fn to_string(&self) -> &str {
        match self {
            UnaryOp::Not => "not",
        }
    }
}

#[derive(Debug)]
pub enum UnaryFnOp {
    Negate,
}

impl UnaryFnOp {
    pub fn fn_name(&self) -> &str {
        match self {
            UnaryFnOp::Negate => "__neg__",
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
