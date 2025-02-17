use std::marker::PhantomData;

/// A trait for reading characters from a source.
pub trait Read<T> {
    const MAX_LOOKAHEAD: usize;

    /// Reads the next value, advances the cursor to the next position.
    fn advance(&mut self) -> Option<T>;

    /// Peeks `n` values ahead.
    fn try_peek(&mut self, n: usize) -> Option<&T>;

    /// Peeks at the next value.
    fn try_peek_next(&mut self) -> Option<&T> {
        self.try_peek(0)
    }

    /// Peeks `n` values ahead, panicking if out of bounds or n > MAX_LOOKAHEAD.
    fn peek(&mut self, n: usize) -> &T {
        self.try_peek(n)
            .expect("Tried to read past the end of the source code.")
    }

    /// Peeks at the next value, panicking if out of bounds or n > MAX_LOOKAHEAD.
    fn peek_next(&mut self) -> &T {
        self.peek(0)
    }

    fn iter(self) -> ReadIterator<T, Self>
    where
        Self: Sized,
    {
        ReadIterator {
            reader: self,
            _marker: PhantomData,
        }
    }
}

/// A wrapper struct to implement Iterator for any type that implements Read.
pub struct ReadIterator<T, R> {
    reader: R,
    _marker: PhantomData<T>,
}

impl<T, R> Iterator for ReadIterator<T, R>
where
    R: Read<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.advance()
    }
}
