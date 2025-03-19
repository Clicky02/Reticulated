use crate::{codegen::util::values::*, wrapper_type};

wrapper_type!(BoolType: IntType);

impl<'ctx> BoolType<'ctx> {
    pub fn const_bool(&mut self, value: bool) -> BoolValue<'ctx> {
        BoolValue::new(self.value.const_int(value as u64, false))
    }
}
