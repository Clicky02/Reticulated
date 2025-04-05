use inkwell::{types::FloatType, values::BasicValue};

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
    parser::{BinaryFnOp, UnaryFnOp},
};

use super::{llvm_resources::LLVMResources, primitive_unalloc};

pub const FLOAT_NAME: &str = "float";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_float_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let float_struct = self.create_struct_type(FLOAT_NAME, vec![self.ctx.f64_type().into()]);
        let float_type = TypeDef::new_prim(FLOAT_NAME, float_struct);

        env.reserve_type_id(FLOAT_ID, true)?;
        env.register_type(FLOAT_NAME, FLOAT_ID, float_type)?;

        Ok(())
    }

    pub fn setup_float_primitive(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.build_free_ptr_fn(FLOAT_ID, primitive_unalloc, env)?;
        self.build_copy_ptr_fn(FLOAT_ID, env)?;

        // Binary
        self.setup_float_add_float(env)?;
        self.setup_float_sub_float(env)?;
        self.setup_float_mul_float(env)?;
        self.setup_float_div_float(env)?;
        self.setup_float_gt_float(env)?;
        self.setup_float_lt_float(env)?;
        self.setup_float_ge_float(env)?;
        self.setup_float_le_float(env)?;

        // Unary
        self.setup_negate_float(env)?;

        // Conversion
        self.setup_float_to_str(res, env)?;

        Ok(())
    }

    fn setup_float_add_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Add.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            FLOAT_ID,
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
            env,
        )
    }

    fn setup_float_sub_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Subtract.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            FLOAT_ID,
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
            env,
        )
    }

    fn setup_float_mul_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Multiply.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            FLOAT_ID,
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
            env,
        )
    }

    fn setup_float_div_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Divide.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            FLOAT_ID,
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
            env,
        )
    }

    fn setup_float_gt_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Greater.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_float_lt_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Less.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_float_ge_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::GreaterEqual.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_float_le_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::LessEqual.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_negate_float(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_unary_fn(
            UnaryFnOp::Negate.fn_name(),
            FLOAT_ID,
            FLOAT_ID,
            |gen, expr| {
                Ok(gen
                    .builder
                    .build_float_neg(expr.into_float_value(), "float_neg")?
                    .as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_float_to_str(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.create_primitive_to_str_fn(FLOAT_ID, "%lf", res, env)
    }

    pub fn prim_float_type(&mut self) -> FloatType<'ctx> {
        self.ctx.f64_type()
    }
}
