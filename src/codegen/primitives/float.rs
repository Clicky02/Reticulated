use inkwell::{types::StructType, values::BasicValue};

use crate::codegen::{
    env::{id::FLOAT_ID, Environment},
    err::GenError,
    CodeGen,
};

pub const FLOAT_NAME: &str = "float";

impl<'ctx> CodeGen<'ctx> {
    pub fn setup_float_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let float_struct = self.ctx.opaque_struct_type(FLOAT_NAME);
        float_struct.set_body(&[self.ctx.f64_type().into()], false);
        env.reserve_type_id(FLOAT_ID, FLOAT_NAME, float_struct)?;

        self.setup_float_add_float(float_struct, env)?;

        Ok(())
    }

    fn setup_float_add_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(
            Some(FLOAT_ID),
            "__add__",
            &[FLOAT_ID, FLOAT_ID],
            FLOAT_ID,
            false,
        )?;
        self.build_primitive_binary_fn(fn_val, float_struct, float_struct, |gen, left, right| {
            Ok(gen
                .builder
                .build_float_add(
                    left.into_float_value(),
                    right.into_float_value(),
                    "float_add",
                )?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
