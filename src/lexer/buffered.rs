use crate::{read::Read, source::ReadSource};

use super::{Lexer, Token};

const LEXER_MAX_LOOKAHEAD: usize = 2;

pub struct BufferedLexer<R: ReadSource> {
    lexer: Lexer<R>,
    lookahead: [Option<Token>; LEXER_MAX_LOOKAHEAD],
}

impl<R: ReadSource> BufferedLexer<R> {
    pub fn new(mut lexer: Lexer<R>) -> Self {
        let mut lookahead = [const { None }; LEXER_MAX_LOOKAHEAD];
        for i in 0..Self::MAX_LOOKAHEAD {
            lookahead[i] = lexer.next();
        }
        Self { lexer, lookahead }
    }
}

impl<R: ReadSource> Read<Token> for BufferedLexer<R> {
    const MAX_LOOKAHEAD: usize = LEXER_MAX_LOOKAHEAD;

    fn advance(&mut self) -> Option<Token> {
        let token = self.lookahead[0].take();
        self.lookahead.rotate_left(1);
        self.lookahead[Self::MAX_LOOKAHEAD - 1] = self.lexer.next();

        token
    }

    fn try_peek(&mut self, n: usize) -> Option<&Token> {
        if n >= Self::MAX_LOOKAHEAD {
            return None; // TODO: Panic?
        }

        self.lookahead[n].as_ref()
    }
}
