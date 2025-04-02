use inkwell::types::StructType;

use crate::codegen::{
    env::{
        id::{BOOL_ID, STR_ID},
        type_def::TypeDef,
        Environment,
    },
    err::GenError,
    CodeGen,
};

use super::{llvm_resources::LLVMResources, primitive_unalloc, TO_STR_FN};

pub const BOOL_NAME: &str = "bool";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_bool_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let bool_struct = self.create_struct_type(BOOL_NAME, vec![self.ctx.bool_type().into()]);
        let bool_type = TypeDef::new_prim(BOOL_NAME, bool_struct);

        env.reserve_type_id(BOOL_ID, true)?;
        env.register_type(BOOL_NAME, BOOL_ID, bool_type)?;

        Ok(())
    }

    pub fn setup_bool_primitive(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let bool_struct = BOOL_ID.get_from(env).ink();

        self.build_free_ptr_fn(BOOL_ID, primitive_unalloc, env)?;
        self.build_copy_ptr_fn(BOOL_ID, env)?;

        // Conversion
        self.setup_bool_to_str(bool_struct, res, env)?;

        Ok(())
    }

    fn setup_bool_to_str(
        &mut self,
        bool_struct: StructType<'ctx>,
        _res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(BOOL_ID), TO_STR_FN, &[BOOL_ID], STR_ID, false)?;

        self.build_unary_fn(fn_val, |gen, param| {
            let prim = gen.extract_primitive(param, bool_struct)?;

            let true_branch = gen.ctx.append_basic_block(fn_val, "true_branch");
            let false_branch = gen.ctx.append_basic_block(fn_val, "false_branch");

            gen.builder.build_conditional_branch(
                prim.into_int_value(),
                true_branch,
                false_branch,
            )?;

            gen.builder.position_at_end(true_branch);
            let true_str = gen.build_str_const("True", env)?;
            gen.builder.build_return(Some(&true_str))?;

            gen.builder.position_at_end(false_branch);
            gen.build_str_const("False", env)
        })
    }
}
