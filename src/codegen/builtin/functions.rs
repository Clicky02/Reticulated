use crate::codegen::{
    env::{
        id::{NONE_ID, STR_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

use super::c_functions::CFunctions;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_functions(
        &mut self,
        cfns: &CFunctions<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.setup_print(cfns, env)?;
        Ok(())
    }

    fn setup_print(
        &mut self,
        cfns: &CFunctions<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let str_type = env.get_type(STR_ID).ink();

        let (print_fn, ..) = env.create_func(None, "print", &[STR_ID], NONE_ID, false)?;

        let entry = self.ctx.append_basic_block(print_fn, "entry");
        self.builder.position_at_end(entry);

        let str_struct_ptr = print_fn.get_nth_param(0).unwrap().into_pointer_value();
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
