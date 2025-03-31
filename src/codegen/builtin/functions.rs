use inkwell::{values::FunctionValue, AddressSpace};

use crate::codegen::{
    env::{
        id::{NONE_ID, STR_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

struct CFunctions<'ctx> {
    snprintf: FunctionValue<'ctx>,
    sscanf: FunctionValue<'ctx>,
    printf: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_functions(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let mut cfns = self.setup_llvm_c_functions(env);
        self.setup_print(env, &mut cfns)?;
        Ok(())
    }

    fn setup_llvm_c_functions(&mut self, env: &mut Environment<'ctx>) -> CFunctions<'ctx> {
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

    fn setup_print(
        &mut self,
        env: &mut Environment<'ctx>,
        cfns: &mut CFunctions<'ctx>,
    ) -> Result<(), GenError> {
        let str_type = env.get_type(STR_ID).ink();

        let (print_fn, ..) = env.create_func(None, "print", &[STR_ID], NONE_ID, false)?;

        let entry = self.ctx.append_basic_block(print_fn, "entry");
        self.builder.position_at_end(entry);

        let str_struct_ptr: inkwell::values::PointerValue<'_> =
            print_fn.get_nth_param(0).unwrap().into_pointer_value();
        let (str_ptr, str_len) = self.build_extract_string(str_struct_ptr, str_type)?;

        let format_spec = self
            .builder
            .build_global_string_ptr("%.*s", "print_string_format")?
            .as_pointer_value();

        self.builder.build_call(
            cfns.printf,
            &[format_spec.into(), str_len.into(), str_ptr.into()],
            "_",
        )?;

        let none_value = self.build_none(env)?;
        self.builder.build_return(Some(&none_value))?;

        Ok(())
    }
}
