use inkwell::{types::StructType, values::BasicValue};

use crate::{
    codegen::{
        env::{id::INT_ID, type_def::TypeDef, Environment},
        err::GenError,
        CodeGen,
    },
    parser::BinaryOp,
};

pub const INT_NAME: &str = "int";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_int_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let int_struct = self.create_struct_type(INT_NAME, vec![self.ctx.i64_type().into()]);
        let int_type = TypeDef::new_prim(INT_NAME, int_struct);

        env.reserve_type_id(INT_ID, true)?;
        env.register_type(INT_NAME, INT_ID, int_type)?;

        Ok(())
    }

    pub fn setup_int_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let int_struct = INT_ID.get_from(env).ink();

        self.build_free_ptr_fn(INT_ID, env)?;
        self.build_copy_ptr_fn(INT_ID, env)?;

        // Binary
        self.setup_int_add_int(int_struct, env)?;
        self.setup_int_sub_int(int_struct, env)?;

        // Unary
        self.setup_negate_int(int_struct, env)?;

        Ok(())
    }

    fn setup_int_add_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(
            Some(INT_ID),
            BinaryOp::Add.fn_name(),
            &[INT_ID, INT_ID],
            Some(INT_ID),
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            int_struct,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_add(left.into_int_value(), right.into_int_value(), "int_add")?
                    .as_basic_value_enum())
            },
        )?;

        Ok(())
    }

    fn setup_int_sub_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(
            Some(INT_ID),
            BinaryOp::Subtract.fn_name(),
            &[INT_ID, INT_ID],
            Some(INT_ID),
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            int_struct,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_sub(left.into_int_value(), right.into_int_value(), "int_sub")?
                    .as_basic_value_enum())
            },
        )?;

        Ok(())
    }

    fn setup_negate_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_val = env.create_func(Some(INT_ID), "__neg__", &[INT_ID], Some(INT_ID), false)?;
        self.build_primitive_unary_fn(fn_val, int_struct, int_struct, |gen, expr| {
            Ok(gen
                .builder
                .build_int_neg(expr.into_int_value(), "int_neg")?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
