use inkwell::{
    values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::codegen::{env::Environment, err::GenError, CodeGen};

pub struct LLVMResources<'ctx> {
    pub str_format_spec: PointerValue<'ctx>,
    pub cstr_format_spec: PointerValue<'ctx>,

    pub scanf: FunctionValue<'ctx>,
    pub get_char: FunctionValue<'ctx>,
    pub sscanf: FunctionValue<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub snprintf: FunctionValue<'ctx>,
    pub realloc: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_llvm_resources(
        &mut self,
        env: &mut Environment<'ctx>,
    ) -> Result<LLVMResources<'ctx>, GenError> {
        let ptr_type = self.ctx.ptr_type(AddressSpace::default());

        let str_format_spec = self
            .builder
            .build_global_string_ptr("%.*s", "print_string_format")?
            .as_pointer_value();

        let cstr_format_spec = self
            .builder
            .build_global_string_ptr("%s", "print_string_format")?
            .as_pointer_value();

        // Add realloc
        let realloc_type = self
            .ptr_type()
            .fn_type(&[ptr_type.into(), self.prim_int_type().into()], false);
        let realloc = env.module().add_function("realloc", realloc_type, None);

        // Add scanf
        let scanf_type = self.ctx.i32_type().fn_type(&[ptr_type.into()], true);
        let scanf = env.module().add_function("scanf", scanf_type, None);

        // Add getchar
        let get_char_type = self.ctx.i32_type().fn_type(&[], false);
        let get_char = env.module().add_function("get_char", get_char_type, None);

        // Add sscanf
        let sscanf_type = self
            .ctx
            .i32_type()
            .fn_type(&[ptr_type.into(), ptr_type.into()], true);
        let sscanf = env.module().add_function("sscanf", sscanf_type, None);

        // Add printf
        let printf_type = self.ctx.i32_type().fn_type(&[ptr_type.into()], true);
        let printf = env.module().add_function("printf", printf_type, None);

        // Add snprintf
        let snprintf_type = self.ctx.i32_type().fn_type(
            &[ptr_type.into(), self.ctx.i64_type().into(), ptr_type.into()],
            true,
        );
        let snprintf = env.module().add_function("snprintf", snprintf_type, None);

        // let ptr_type = self.ctx.ptr_type(AddressSpace::default());
        // let fd_type = self.ctx.opaque_struct_type("FILE");
        // let stdin_ptr = env.module.add_global(fd_type, None, "stdin").as_pointer_value();

        Ok(LLVMResources {
            str_format_spec,
            cstr_format_spec,

            scanf,
            get_char,
            sscanf,
            printf,
            snprintf,
            realloc,
        })
    }

    pub(super) fn build_get_string_size(
        &mut self,
        format_spec: PointerValue<'ctx>,
        primitive: BasicValueEnum<'ctx>,
        res: &LLVMResources<'ctx>,
    ) -> Result<IntValue<'ctx>, GenError> {
        let cstr_size = self
            .builder
            .build_call(
                res.snprintf,
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
