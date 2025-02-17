use crate::read::Read;

use super::{KeywordKind, OperatorKind, Token, TokenKind};

pub trait ReadTokens: Read<Token> {
    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        let next_token = self.advance();
        if let Some(next_token) = next_token {
            if next_token.kind == kind {
                Ok(())
            } else {
                Err(format!(
                    "Unexpected token `{}` at {}, expected {}.",
                    next_token.kind, next_token.span.start, kind
                )) // TODO: Real error, better formatting
            }
        } else {
            Err("Unexpected end of input.".into())
        }
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        let next_token = self.try_peek_next();
        if let Some(next_token) = next_token {
            if next_token.kind == kind {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn expect_operator(&mut self, kind: OperatorKind) -> Result<(), String> {
        self.expect(TokenKind::Operator(kind))
    }

    fn expect_keyword(&mut self, kind: KeywordKind) -> Result<(), String> {
        self.expect(TokenKind::Keyword(kind))
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        let token = self.advance();
        match token {
            Some(Token {
                kind: TokenKind::Identifier(identifier),
                ..
            }) => Ok(identifier),
            Some(token) => Err(format!("Expected identifier, found {:?}", token)),
            _ => Err("Unexpected end of input.".into()),
        }
    }
}

impl<R: Read<Token>> ReadTokens for R {}
