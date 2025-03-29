use inkwell::{types::StructType, values::BasicValue};

use crate::{
    codegen::{
        env::{
            id::{BOOL_ID, INT_ID},
            type_def::TypeDef,
            Environment,
        },
        err::GenError,
        CodeGen,
    },
    parser::BinaryOp,
};

use super::primitive_unalloc;

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

        self.build_free_ptr_fn(INT_ID, primitive_unalloc, env)?;
        self.build_copy_ptr_fn(INT_ID, env)?;

        // Binary
        self.setup_int_add_int(int_struct, env)?;
        self.setup_int_sub_int(int_struct, env)?;
        self.setup_int_mul_int(int_struct, env)?;
        self.setup_int_div_int(int_struct, env)?;
        self.setup_int_gt_int(int_struct, env)?;
        self.setup_int_lt_int(int_struct, env)?;
        self.setup_int_ge_int(int_struct, env)?;
        self.setup_int_le_int(int_struct, env)?;

        // Unary
        self.setup_negate_int(int_struct, env)?;

        Ok(())
    }

    fn setup_int_add_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Add.fn_name(),
            &[INT_ID, INT_ID],
            INT_ID,
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
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Subtract.fn_name(),
            &[INT_ID, INT_ID],
            INT_ID,
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

    fn setup_int_mul_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Multiply.fn_name(),
            &[INT_ID, INT_ID],
            INT_ID,
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
                    .build_int_mul(left.into_int_value(), right.into_int_value(), "int_mul")?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_int_div_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Divide.fn_name(),
            &[INT_ID, INT_ID],
            INT_ID,
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
                    .build_int_signed_div(left.into_int_value(), right.into_int_value(), "int_div")?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_int_gt_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Greater.fn_name(),
            &[INT_ID, INT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::SGT,
                        left.into_int_value(),
                        right.into_int_value(),
                        "int_gt",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_int_lt_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::Less.fn_name(),
            &[INT_ID, INT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::SLT,
                        left.into_int_value(),
                        right.into_int_value(),
                        "int_lt",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_int_ge_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::GreaterEqual.fn_name(),
            &[INT_ID, INT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::SGE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "int_ge",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_int_le_int(
        &mut self,
        int_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(INT_ID),
            BinaryOp::LessEqual.fn_name(),
            &[INT_ID, INT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            int_struct,
            int_struct,
            env.get_type(BOOL_ID).ink(),
            |gen: &mut CodeGen<'ctx>, left, right| {
                Ok(gen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::SLE,
                        left.into_int_value(),
                        right.into_int_value(),
                        "int_le",
                    )?
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
        let (fn_val, ..) = env.create_func(Some(INT_ID), "__neg__", &[INT_ID], INT_ID, false)?;
        self.build_primitive_unary_fn(fn_val, int_struct, int_struct, |gen, expr| {
            Ok(gen
                .builder
                .build_int_neg(expr.into_int_value(), "int_neg")?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
