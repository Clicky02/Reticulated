use crate::{codegen::util::values::*, wrapper_type};

wrapper_type!(FloatType: FloatType);

impl<'ctx> FloatType<'ctx> {
    pub fn const_float(&mut self, value: f64) -> FloatValue<'ctx> {
        FloatValue::new(self.value.const_float(value))
    }
}
