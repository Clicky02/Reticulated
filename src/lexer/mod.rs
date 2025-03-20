pub use buffered::*;
pub use read_tokens::*;
pub use token::*;

use crate::source::{Position, ReadSource};

mod buffered;
mod read_tokens;
mod token;

#[derive(Debug)]
pub struct LexError(pub String);

/// A lexer for the source code.
///
/// The last token exported by a Lexer will always be a EOF token.
/// Other parts of the codebase rely on this.
pub struct Lexer<R: ReadSource> {
    source: R,
    token_start: Position,
    reached_eof: bool,
}

impl<R: ReadSource> Lexer<R> {
    pub fn new(source: R) -> Self {
        let token_start = source.pos();

        Lexer {
            source,
            token_start,
            reached_eof: false,
        }
    }

    /// Advances the input by one character if the next character matches the expected character.
    fn consume_expected(&mut self, expected: char) -> bool {
        if let Some(&ch) = self.source.try_peek_next() {
            if ch == expected {
                self.source.advance();
                return true;
            }
        }

        false
    }

    /// Advances the input while the predicate returns true.
    fn consume_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(&ch) = self.source.try_peek_next() {
            if predicate(ch) {
                self.source.advance();
            } else {
                break;
            }
        }
    }

    /// Consumes a string literal (starting after the open quote)
    fn consume_string(&mut self) -> Result<TokenKind, LexError> {
        while let Some(ch) = self.source.advance() {
            if ch == '\\' {
                // Skip the escaped character
                self.source.advance();
            } else if ch == '"' {
                let start = self.token_start.index() + 1;
                let end = self.source.pos().index() - 1;

                return Ok(TokenKind::Literal(LiteralKind::String(
                    self.source.range(start..end),
                )));
            }
        }

        Err(LexError(
            "String Literal was not closed with a quote.".into(),
        ))
    }

    /// Consumes an identifier (starting with an alphabetic character)
    fn consume_identifier(&mut self) -> TokenKind {
        self.consume_while(|ch| ch.is_alphanumeric() || ch == '_');
        let text = self.source.range(self.token_start..self.source.pos());

        match &text[..] {
            "if" => TokenKind::Keyword(KeywordKind::If),
            "else" => TokenKind::Keyword(KeywordKind::Else),
            "def" => TokenKind::Keyword(KeywordKind::Def),
            "extern" => TokenKind::Keyword(KeywordKind::Extern),
            "for" => TokenKind::Keyword(KeywordKind::For),
            "while" => TokenKind::Keyword(KeywordKind::While),
            "return" => TokenKind::Keyword(KeywordKind::Return),

            // TODO: Should these be keywords?
            "or" => TokenKind::Operator(OperatorKind::Or),
            "and" => TokenKind::Operator(OperatorKind::And),
            "not" => TokenKind::Operator(OperatorKind::Not),

            "True" => TokenKind::Literal(LiteralKind::Boolean(true)),
            "False" => TokenKind::Literal(LiteralKind::Boolean(false)),

            _ => TokenKind::Identifier(text),
        }
    }

    /// Consumes a number (starting with a digit)
    fn consume_number(&mut self) -> TokenKind {
        let mut is_float = false;

        self.consume_while(|ch| ch.is_digit(10));
        if self.consume_expected('.') {
            is_float = true;
            self.consume_while(|ch| ch.is_digit(10));
        }

        if is_float {
            TokenKind::Literal(LiteralKind::Float(
                self.source
                    .range(self.token_start..self.source.pos())
                    .parse()
                    .unwrap(),
            ))
        } else {
            TokenKind::Literal(LiteralKind::Integer(
                self.source
                    .range(self.token_start..self.source.pos())
                    .parse()
                    .unwrap(),
            ))
        }
    }

    /// Begins a token, should be called before advancing.
    fn start_token(&mut self) {
        self.token_start = self.source.pos();
    }
}

impl<R: ReadSource> Iterator for Lexer<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! followed_by {
            ($exp_char:literal => $token:expr) => {
                if self.consume_expected($exp_char) {
                    $token
                } else {
                    TokenKind::Invalid
                }
            };
            ($exp_char:literal => $token:expr, _ => $default:expr,) => {
                if self.consume_expected($exp_char) {
                    $token
                } else {
                    $default
                }
            };
            ($($exp_char:literal => $token:expr,)+ _ => $default:expr,) => {
                if let Some(next) = self.source.try_peek_next() {
                    match next {
                        $($exp_char => {
                            self.source.advance();
                            $token
                        }),+
                        _ => $default,
                    }
                } else {
                    $default
                }
            };
        }

        if self.reached_eof {
            return None;
        }

        self.consume_while(char::is_whitespace);

        self.start_token();
        let ch = self.source.advance();

        let kind = match ch {
            Some(ch) => match ch {
                '(' => TokenKind::OpenParenthesis,
                ')' => TokenKind::CloseParenthesis,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                ',' => TokenKind::Comma,
                ';' => TokenKind::SemiColon,
                ':' => TokenKind::Colon,

                '+' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::AddAssign),
                    _ => TokenKind::Operator(OperatorKind::Add),
                ),
                '-' => followed_by!(
                    '>' => TokenKind::Arrow,
                    '=' => TokenKind::Operator(OperatorKind::SubtractAssign),
                    _ => TokenKind::Operator(OperatorKind::Subtract),
                ),
                '*' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::MultiplyAssign),
                    _ => TokenKind::Operator(OperatorKind::Multiply),
                ),
                '/' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::DivideAssign),
                    _ => TokenKind::Operator(OperatorKind::Divide),
                ),
                '%' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::ModuloAssign),
                    _ => TokenKind::Operator(OperatorKind::Modulo),
                ),

                '=' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::Equal),
                    _ => TokenKind::Operator(OperatorKind::Assign),
                ),
                '!' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::NotEqual),
                    _ => TokenKind::Operator(OperatorKind::Not),
                ),
                '<' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::LessThanOrEqual),
                    _ => TokenKind::Operator(OperatorKind::LessThan),
                ),
                '>' => followed_by!(
                    '=' => TokenKind::Operator(OperatorKind::GreaterThanOrEqual),
                    _ => TokenKind::Operator(OperatorKind::GreaterThan),
                ),
                '&' => followed_by!('&' => TokenKind::Operator(OperatorKind::And)),
                '|' => followed_by!('|' => TokenKind::Operator(OperatorKind::Or)),

                '"' => self.consume_string().unwrap(),
                _ if ch.is_alphabetic() => self.consume_identifier(),
                _ if ch.is_digit(10) => self.consume_number(),
                _ => TokenKind::Invalid,
            },
            None => {
                self.reached_eof = true;
                TokenKind::EOF
            }
        };

        Some(Token {
            kind,
            span: Span {
                start: self.token_start,
                end: self.source.pos(),
            },
        })
    }
}
