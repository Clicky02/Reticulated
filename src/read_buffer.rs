use crate::read::Read;

/// A wrapper that implements Read<T> for a Vec<T>
pub struct ReadBuffer<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> ReadBuffer<T> {
    pub fn new(buffer: Vec<T>) -> Self {
        Self { buffer, index: 0 }
    }
}

impl<T: Clone> Read<T> for ReadBuffer<T> {
    const MAX_LOOKAHEAD: usize = usize::MAX;

    fn advance(&mut self) -> Option<T> {
        if self.index >= self.buffer.len() {
            return None;
        }

        let value = self.buffer[self.index].clone();
        self.index += 1;
        Some(value)
    }

    fn try_peek(&mut self, n: usize) -> Option<&T> {
        if n + self.index >= self.buffer.len() {
            return None;
        }

        Some(&self.buffer[n + self.index])
    }
}
