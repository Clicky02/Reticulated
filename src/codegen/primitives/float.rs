use inkwell::{types::StructType, values::BasicValue};

use crate::{
    codegen::{
        env::{
            id::{BOOL_ID, FLOAT_ID},
            type_def::TypeDef,
            Environment,
        },
        err::GenError,
        CodeGen,
    },
    parser::{BinaryOp, UnaryOp},
};

pub const FLOAT_NAME: &str = "float";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_float_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let float_struct = self.create_struct_type(FLOAT_NAME, vec![self.ctx.f64_type().into()]);
        let float_type = TypeDef::new_prim(FLOAT_NAME, float_struct);

        env.reserve_type_id(FLOAT_ID, true)?;
        env.register_type(FLOAT_NAME, FLOAT_ID, float_type)?;

        Ok(())
    }

    pub fn setup_float_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let float_struct = FLOAT_ID.get_from(env).ink();

        self.build_free_ptr_fn(FLOAT_ID, env)?;
        self.build_copy_ptr_fn(FLOAT_ID, env)?;

        // Binary
        self.setup_float_add_float(float_struct, env)?;
        self.setup_float_sub_float(float_struct, env)?;
        self.setup_float_mul_float(float_struct, env)?;
        self.setup_float_div_float(float_struct, env)?;
        self.setup_float_gt_float(float_struct, env)?;
        self.setup_float_lt_float(float_struct, env)?;
        self.setup_float_ge_float(float_struct, env)?;
        self.setup_float_le_float(float_struct, env)?;

        // Unary
        self.setup_negate_float(float_struct, env)?;

        Ok(())
    }

    fn setup_float_add_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Add.fn_name(),
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
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Subtract.fn_name(),
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

    fn setup_float_mul_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Multiply.fn_name(),
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
                    .build_float_mul(
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_mul",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_float_div_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Divide.fn_name(),
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
                    .build_float_div(
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_div",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_float_gt_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Greater.fn_name(),
            &[FLOAT_ID, FLOAT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_gt",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_float_lt_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::Less.fn_name(),
            &[FLOAT_ID, FLOAT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLT,
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_lt",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_float_ge_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::GreaterEqual.fn_name(),
            &[FLOAT_ID, FLOAT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_ge",
                    )?
                    .as_basic_value_enum())
            },
        )?;
        Ok(())
    }

    fn setup_float_le_float(
        &mut self,
        float_struct: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            BinaryOp::LessEqual.fn_name(),
            &[FLOAT_ID, FLOAT_ID],
            BOOL_ID,
            false,
        )?;
        self.build_primitive_binary_fn(
            fn_val,
            float_struct,
            float_struct,
            env.get_type(BOOL_ID).ink(),
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLE,
                        left.into_float_value(),
                        right.into_float_value(),
                        "float_le",
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
        let (fn_val, ..) = env.create_func(
            Some(FLOAT_ID),
            UnaryOp::Negate.fn_name(),
            &[FLOAT_ID],
            FLOAT_ID,
            false,
        )?;
        self.build_primitive_unary_fn(fn_val, float_struct, float_struct, |gen, expr| {
            Ok(gen
                .builder
                .build_float_neg(expr.into_float_value(), "float_neg")?
                .as_basic_value_enum())
        })?;

        Ok(())
    }
}
