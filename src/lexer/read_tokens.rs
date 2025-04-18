use anyhow::{anyhow, Result};

use crate::read::Read;

use super::{KeywordKind, OperatorKind, Token, TokenKind};

pub trait ReadTokens: Read<Token> {
    fn expect(&mut self, kind: TokenKind) -> Result<()> {
        let next_token = self.advance();
        if let Some(next_token) = next_token {
            if next_token.kind == kind {
                Ok(())
            } else {
                Err(anyhow!(
                    "Unexpected token `{}` at {}, expected {}.",
                    next_token.kind,
                    next_token.span.start,
                    kind
                ))
            }
        } else {
            Err(anyhow!("Unexpected end of input."))
        }
    }

    /// Without advancing, returns whether the next token matches the given token
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

    fn expect_operator(&mut self, kind: OperatorKind) -> Result<()> {
        self.expect(TokenKind::Operator(kind))
    }

    fn expect_keyword(&mut self, kind: KeywordKind) -> Result<()> {
        self.expect(TokenKind::Keyword(kind))
    }

    fn expect_identifier(&mut self) -> Result<String> {
        let token = self.advance();
        match token {
            Some(Token {
                kind: TokenKind::Identifier(identifier),
                ..
            }) => Ok(identifier),
            Some(token) => Err(anyhow!(
                "Expected identifier at {}, found {}",
                token.span,
                token.kind
            )),
            _ => Err(anyhow!("Unexpected end of input.")),
        }
    }
}

impl<R: Read<Token>> ReadTokens for R {}
