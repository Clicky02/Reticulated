use inkwell::{types::StructType, values::BasicValue};

use crate::codegen::{
    env::{id::INT_ID, Environment},
    err::GenError,
    CodeGen,
};

pub const INT_NAME: &str = "int";

impl<'ctx> CodeGen<'ctx> {
    pub fn setup_int_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let int_struct = self.ctx.opaque_struct_type(INT_NAME);
        int_struct.set_body(&[self.ctx.i64_type().into()], false);
        env.reserve_type_id(INT_ID, INT_NAME, int_struct)?;

        self.setup_int_add_int(int_struct, env)?;

        Ok(())
    }

    fn setup_int_add_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(Some(INT_ID), "__add__", &[INT_ID, INT_ID], INT_ID, false)?;
        self.build_primitive_binary_fn(fn_val, int_struct, int_struct, |gen, left, right| {
            Ok(gen
                .builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "int_add")?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
