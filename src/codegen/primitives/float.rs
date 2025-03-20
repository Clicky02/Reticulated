use inkwell::{types::StructType, values::BasicValue};

use crate::codegen::{
    env::{id::FLOAT_ID, Environment},
    err::GenError,
    CodeGen,
};

pub const FLOAT_NAME: &str = "float";

impl<'ctx> CodeGen<'ctx> {
    pub fn setup_float_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let float_struct = self.create_struct_type(FLOAT_NAME, vec![self.ctx.f64_type().into()]);
        env.reserve_type_id(FLOAT_ID, FLOAT_NAME, float_struct)?;

        // Binary
        self.setup_float_add_float(float_struct, env)?;
        self.setup_float_sub_float(float_struct, env)?;

        // Unary
        self.setup_negate_float(float_struct, env)?;

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
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            float_struct,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_add(
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_add",
                    )?
                    .as_basic_value_enum())
            },
        )?;

        Ok(())
    }

    fn setup_float_sub_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(
            Some(FLOAT_ID),
            "__sub__",
            &[FLOAT_ID, FLOAT_ID],
            FLOAT_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            float_struct,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_sub(
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_sub",
                    )?
                    .as_basic_value_enum())
            },
        )?;

        Ok(())
    }

    fn setup_negate_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(Some(FLOAT_ID), "__neg__", &[FLOAT_ID], FLOAT_ID, false)?;
        self.build_primitive_unary_fn(fn_val, float_struct, float_struct, |gen, expr| {
            Ok(gen
                .builder
                .build_float_neg(expr.into_float_value(), "float_neg")?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
