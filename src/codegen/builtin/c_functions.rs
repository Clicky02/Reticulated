use inkwell::{
    values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::codegen::{env::Environment, err::GenError, CodeGen};

pub struct CFunctions<'ctx> {
    pub snprintf: FunctionValue<'ctx>,
    pub sscanf: FunctionValue<'ctx>,
    pub printf: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_llvm_c_functions(
        &mut self,
        env: &mut Environment<'ctx>,
    ) -> CFunctions<'ctx> {
        let ptr_type = self.ctx.ptr_type(AddressSpace::default());

        // Add snprintf
        let snprintf_type = self.ctx.i32_type().fn_type(
            &[ptr_type.into(), self.ctx.i64_type().into(), ptr_type.into()],
            true,
        );
        let snprintf = env.module().add_function("snprintf", snprintf_type, None);

        let sscanf_type = self
            .ctx
            .i32_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], true);
        let sscanf = env.module().add_function("sscanf", sscanf_type, None);

        // Add printf
        let printf_type = self.ctx.i32_type().fn_type(&[ptr_type.into()], true);
        let printf = env.module().add_function("printf", printf_type, None);

        CFunctions {
            snprintf,
            sscanf,
            printf,
        }
    }

    pub(super) fn build_get_string_size(
        &mut self,
        format_spec: PointerValue<'ctx>,
        primitive: BasicValueEnum<'ctx>,
        cfns: &CFunctions<'ctx>,
    ) -> Result<IntValue<'ctx>, GenError> {
        let cstr_size = self
            .builder
            .build_call(
                cfns.snprintf,
                &[
                    self.str_ptr_type().const_null().into(),
                    self.len_type().const_int(0, false).into(),
                    format_spec.into(),
                    primitive.into(),
                ],
                "snprintf_size_i32",
            )?
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();

        let cstr_size = self
            .builder
            .build_int_cast(cstr_size, self.len_type(), "snprintf_size")?;

        Ok(cstr_size)
    }
}
