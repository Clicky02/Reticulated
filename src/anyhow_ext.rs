use anyhow::Context;

use crate::source::Position;

pub trait AnyhowResultExt<T> {
    fn parsing_ctx(self, parse_obj: &str, pos: Position) -> anyhow::Result<T, anyhow::Error>;
}

impl<T> AnyhowResultExt<T> for anyhow::Result<T, anyhow::Error> {
    fn parsing_ctx(self, parse_obj: &str, pos: Position) -> anyhow::Result<T, anyhow::Error> {
        self.with_context(|| format!("Failed to parse {} at {}", parse_obj, pos))
    }
}
