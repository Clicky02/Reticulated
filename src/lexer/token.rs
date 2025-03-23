use crate::source::Position;

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.kind, self.span)
    }
}
#[derive(Clone, Debug)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    Identifier(String),
    Literal(LiteralKind),
    Operator(OperatorKind),
    Keyword(KeywordKind),

    OpenParenthesis,
    CloseParenthesis,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,

    Comma,
    SemiColon,
    Colon,
    Arrow,

    EOF,

    Invalid,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "Identifier(\"{}\")", s),
            TokenKind::Literal(l) => write!(f, "Literal({})", l),
            TokenKind::Operator(o) => write!(f, "Operator({:?})", o),
            TokenKind::Keyword(k) => write!(f, "Keyword({:?})", k),
            TokenKind::OpenParenthesis => write!(f, "OpenParenthesis"),
            TokenKind::CloseParenthesis => write!(f, "CloseParenthesis"),
            TokenKind::OpenBracket => write!(f, "OpenBracket"),
            TokenKind::CloseBracket => write!(f, "CloseBracket"),
            TokenKind::OpenBrace => write!(f, "OpenBrace"),
            TokenKind::CloseBrace => write!(f, "CloseBrace"),
            TokenKind::Comma => write!(f, "Comma"),
            TokenKind::SemiColon => write!(f, "SemiColon"),
            TokenKind::Colon => write!(f, "Colon"),
            TokenKind::Arrow => write!(f, "Arrow"),
            TokenKind::EOF => write!(f, "EOF"),
            TokenKind::Invalid => write!(f, "Invalid"),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralKind {
    Float(f64),
    Integer(i64),
    String(String),
    Boolean(bool),
}

impl std::fmt::Display for LiteralKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralKind::Float(v) => write!(f, "{}", v),
            LiteralKind::Integer(v) => write!(f, "{}", v),
            LiteralKind::String(v) => write!(f, "\"{}\"", v),
            LiteralKind::Boolean(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorKind {
    Assign,

    Add,
    AddAssign,
    Subtract,
    SubtractAssign,
    Multiply,
    MultiplyAssign,
    Divide,
    DivideAssign,
    Modulo,
    ModuloAssign,

    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Not,
}

impl std::fmt::Display for OperatorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorKind::Assign => write!(f, "="),
            OperatorKind::Add => write!(f, "+"),
            OperatorKind::AddAssign => write!(f, "+="),
            OperatorKind::Subtract => write!(f, "-"),
            OperatorKind::SubtractAssign => write!(f, "-="),
            OperatorKind::Multiply => write!(f, "*"),
            OperatorKind::MultiplyAssign => write!(f, "*="),
            OperatorKind::Divide => write!(f, "/"),
            OperatorKind::DivideAssign => write!(f, "/="),
            OperatorKind::Modulo => write!(f, "%"),
            OperatorKind::ModuloAssign => write!(f, "%="),
            OperatorKind::Equal => write!(f, "=="),
            OperatorKind::NotEqual => write!(f, "!="),
            OperatorKind::LessThan => write!(f, "<"),
            OperatorKind::LessThanOrEqual => write!(f, "<="),
            OperatorKind::GreaterThan => write!(f, ">"),
            OperatorKind::GreaterThanOrEqual => write!(f, ">="),
            OperatorKind::And => write!(f, "and"),
            OperatorKind::Or => write!(f, "or"),
            OperatorKind::Not => write!(f, "not"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeywordKind {
    If,
    Else,
    For,
    While,
    Def,
    Return,
    Extern,
    Struct,
}
