use inkwell::{
    basic_block::BasicBlock,
    builder::{self, Builder as InkwellBuilder, BuilderError},
};

use super::values::IntValue;

pub struct Builder<'ctx> {
    builder: InkwellBuilder<'ctx>,
}

impl<'ctx> Builder<'ctx> {
    pub(super) fn new(builder: InkwellBuilder<'ctx>) -> Self {
        Self { builder }
    }

    pub fn position_at_end(&self, block: BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }

    pub fn build_int_add(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, BuilderError> {
        self.builder.build_int_add(*lhs, *rhs, name).map(IntValue::from)
    }
}
