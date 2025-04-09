use inkwell::{types::IntType, values::BasicValue};

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
    parser::{BinaryFnOp, UnaryFnOp},
};

use super::{llvm_resources::LLVMResources, primitive_unalloc};

pub const INT_NAME: &str = "int";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_int_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let int_struct = self.create_struct_type(INT_NAME, vec![self.ctx.i64_type().into()]);
        let int_type = TypeDef::new_prim(INT_NAME, int_struct);

        env.reserve_type_id(INT_ID, true)?;
        env.register_type(INT_NAME, INT_ID, int_type)?;

        Ok(())
    }

    pub fn setup_int_primitive(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.build_free_ptr_fn(INT_ID, primitive_unalloc, env)?;
        self.build_copy_ptr_fn(INT_ID, env)?;
        self.build_get_reference_count_fn(INT_ID, env)?;

        // Binary
        self.setup_int_add_int(env)?;
        self.setup_int_sub_int(env)?;
        self.setup_int_mul_int(env)?;
        self.setup_int_div_int(env)?;
        self.setup_int_gt_int(env)?;
        self.setup_int_lt_int(env)?;
        self.setup_int_ge_int(env)?;
        self.setup_int_le_int(env)?;
        self.setup_int_pow_int(env)?;

        // Unary
        self.setup_negate_int(env)?;

        // Conversion
        self.setup_int_to_str(res, env)?;

        Ok(())
    }

    fn setup_int_add_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Add.fn_name(),
            INT_ID,
            INT_ID,
            INT_ID,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_add(left.into_int_value(), right.into_int_value(), "int_add")?
                    .as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_int_sub_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Subtract.fn_name(),
            INT_ID,
            INT_ID,
            INT_ID,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_sub(left.into_int_value(), right.into_int_value(), "int_sub")?
                    .as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_int_mul_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Multiply.fn_name(),
            INT_ID,
            INT_ID,
            INT_ID,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_mul(left.into_int_value(), right.into_int_value(), "int_mul")?
                    .as_basic_value_enum())
            },
            env,
        )?;
        Ok(())
    }

    fn setup_int_div_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Divide.fn_name(),
            INT_ID,
            INT_ID,
            INT_ID,
            |gen, left, right| {
                Ok(gen
                    .builder
                    .build_int_signed_div(left.into_int_value(), right.into_int_value(), "int_div")?
                    .as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_int_pow_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Exponentiate.fn_name(),
            INT_ID,
            INT_ID,
            INT_ID,
            |gen, left, right| {
                // Code this is based off of
                // int power(int base, int exponent) {
                //     int result = 1; // entry
                //     while (exponent > 0) { // loop_block
                //         // loop body
                //         if (exponent % 2 == 1) { // odd_exponent
                //             result *= base;
                //         }
                //         //loop continue
                //         base *= base; // Square the base
                //         exponent /= 2; // Divide exponent by 2
                //     }
                //     // exit
                //     return result;
                // }

                let entry_block = gen.builder.get_insert_block().unwrap();
                let fn_val = entry_block.get_parent().unwrap();

                let loop_block = gen.ctx.append_basic_block(fn_val, "loop");
                let loop_body = gen.ctx.append_basic_block(fn_val, "loop_body");
                let odd_exp_block = gen.ctx.append_basic_block(fn_val, "odd_exp");
                let cont_block = gen.ctx.append_basic_block(fn_val, "loop_continue");
                let exit_block = gen.ctx.append_basic_block(fn_val, "exit");

                gen.builder.position_at_end(entry_block);
                let result = gen.builder.build_alloca(gen.prim_int_type(), "result")?;
                gen.builder
                    .build_store(result, gen.prim_int_type().const_int(1, false))?;

                let base_var = gen.builder.build_alloca(gen.prim_int_type(), "base")?;
                gen.builder.build_store(base_var, left.into_int_value())?;

                let exp_var = gen.builder.build_alloca(gen.prim_int_type(), "exp")?;
                gen.builder.build_store(exp_var, right.into_int_value())?;
                gen.builder.build_unconditional_branch(loop_block)?;

                gen.builder.position_at_end(loop_block);
                let current_exp =
                    gen.builder
                        .build_load(gen.prim_int_type(), exp_var, "current_exp")?;
                let exp_gt_zero = gen.builder.build_int_compare(
                    inkwell::IntPredicate::SGT,
                    current_exp.into_int_value(),
                    gen.prim_int_type().const_int(0, false),
                    "exp_gt_zero",
                )?;
                gen.builder
                    .build_conditional_branch(exp_gt_zero, loop_body, exit_block)?;

                gen.builder.position_at_end(loop_body);
                // let current_exp =
                //     gen.builder
                //         .build_load(gen.prim_int_type(), exp_var, "current_exp")?;
                let exp_mod_2 = gen.builder.build_int_signed_rem(
                    current_exp.into_int_value(),
                    gen.prim_int_type().const_int(2, false),
                    "exp_mod_2",
                )?;
                let is_odd = gen.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    exp_mod_2,
                    gen.prim_int_type().const_int(1, false),
                    "is_odd",
                )?;
                let current_base =
                    gen.builder
                        .build_load(gen.prim_int_type(), base_var, "current_base")?;
                let current_result =
                    gen.builder
                        .build_load(gen.prim_int_type(), result, "current_result")?;
                gen.builder
                    .build_conditional_branch(is_odd, odd_exp_block, cont_block)?;

                gen.builder.position_at_end(odd_exp_block);
                let new_result = gen.builder.build_int_mul(
                    current_result.into_int_value(),
                    current_base.into_int_value(),
                    "new_result",
                )?;
                gen.builder.build_store(result, new_result)?;
                gen.builder.build_unconditional_branch(cont_block)?;

                gen.builder.position_at_end(cont_block);
                let new_base = gen.builder.build_int_mul(
                    current_base.into_int_value(),
                    current_base.into_int_value(),
                    "new_base",
                )?;
                gen.builder.build_store(base_var, new_base)?;

                let new_exp = gen.builder.build_int_signed_div(
                    current_exp.into_int_value(),
                    gen.prim_int_type().const_int(2, false),
                    "new_exp",
                )?;
                gen.builder.build_store(exp_var, new_exp)?;
                gen.builder.build_unconditional_branch(loop_block)?;

                gen.builder.position_at_end(exit_block);
                let final_result =
                    gen.builder
                        .build_load(gen.prim_int_type(), result, "final_result")?;
                Ok(final_result.as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_int_gt_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Greater.fn_name(),
            INT_ID,
            INT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_int_lt_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::Less.fn_name(),
            INT_ID,
            INT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_int_ge_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::GreaterEqual.fn_name(),
            INT_ID,
            INT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_int_le_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_binary_fn(
            BinaryFnOp::LessEqual.fn_name(),
            INT_ID,
            INT_ID,
            BOOL_ID,
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
            env,
        )
    }

    fn setup_negate_int(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.create_primitive_unary_fn(
            UnaryFnOp::Negate.fn_name(),
            INT_ID,
            INT_ID,
            |gen, expr| {
                Ok(gen
                    .builder
                    .build_int_neg(expr.into_int_value(), "int_neg")?
                    .as_basic_value_enum())
            },
            env,
        )
    }

    fn setup_int_to_str(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.create_primitive_to_str_fn(INT_ID, "%ld", res, env)
    }

    pub fn prim_int_type(&self) -> IntType<'ctx> {
        self.ctx.i64_type()
    }
}
