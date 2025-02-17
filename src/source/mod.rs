use std::{ops::RangeBounds, str::Chars};

mod position;

pub use position::*;

use crate::read::Read;

/// A trait for reading characters from a source.
pub trait ReadSource: Read<char> {
    /// Returns the current position in the source.
    fn pos(&self) -> Position;

    /// Returns a range of characters from the source code.
    /// TODO: Do I keep the Into<usize> generic or just use usize and force the conversion?
    fn range<T: Clone + Into<usize>, R: RangeBounds<T>>(&mut self, idx: R) -> String;
}

// A cursor over the source code
pub struct SourceCursor<'a> {
    source: &'a str,
    position: Position,
    chars: Chars<'a>,
    lookahead: [Option<char>; SourceCursor::MAX_LOOKAHEAD],
}

impl<'a> SourceCursor<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let mut lookahead = [None; Self::MAX_LOOKAHEAD];
        for i in 0..Self::MAX_LOOKAHEAD {
            lookahead[i] = chars.next();
        }
        Self {
            source,
            position: Position::new(),
            chars,
            lookahead,
        }
    }

    fn advance_lookahead(&mut self) {
        self.lookahead.rotate_left(1);
        self.lookahead[Self::MAX_LOOKAHEAD - 1] = self.chars.next();
    }
}

impl<'a> Read<char> for SourceCursor<'a> {
    const MAX_LOOKAHEAD: usize = 2;

    fn advance(&mut self) -> Option<char> {
        let ch = self.lookahead[0];
        self.position.advance(ch);
        self.advance_lookahead();

        ch
    }

    fn try_peek(&mut self, n: usize) -> Option<&char> {
        if n > Self::MAX_LOOKAHEAD {
            return None; // TODO: Panic?
        }

        self.lookahead[n].as_ref()
    }
}

impl<'a> ReadSource for SourceCursor<'a> {
    fn pos(&self) -> Position {
        self.position
    }

    fn range<T: Clone + Into<usize>, R: RangeBounds<T>>(&mut self, idx: R) -> String {
        let start = match idx.start_bound() {
            std::ops::Bound::Included(n) => n.clone().into(),
            std::ops::Bound::Excluded(n) => n.clone().into() + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match idx.end_bound() {
            std::ops::Bound::Included(n) => n.clone().into() + 1,
            std::ops::Bound::Excluded(n) => n.clone().into(),
            std::ops::Bound::Unbounded => 0,
        };

        self.source[start..end].into()
    }
}
