use inkwell::types::BasicType;

use crate::{codegen::util::values::*, wrapper_type};

use super::{FunctionType, TypeEnum};

wrapper_type!(IntType: IntType);

impl<'ctx> IntType<'ctx> {
    pub fn const_int(&mut self, value: i64) -> IntValue<'ctx> {
        IntValue::new(self.value.const_int(value as u64, false))
    }

    pub fn fn_type(
        &mut self,
        param_types: Vec<TypeEnum<'ctx>>,
        is_var_args: bool,
    ) -> FunctionType<'ctx> {
        let ink_param_types = param_types.iter().map(|t| t.try_as_inkwell_basic()).collect::<Result<Vec<_>, _>>();
        let ink_fn_type = self.value.fn_type(&ink_param_types?, is_var_args);
        FunctionType::new(self.value.fn_type(
            &,
            is_var_args,
        ),
    param_types,
Some())
    }
}
