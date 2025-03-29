use crate::codegen::{
    env::{id::BOOL_ID, type_def::TypeDef, Environment},
    err::GenError,
    CodeGen,
};

use super::primitive_unalloc;

pub const BOOL_NAME: &str = "bool";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_bool_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let bool_struct = self.create_struct_type(BOOL_NAME, vec![self.ctx.bool_type().into()]);
        let bool_type = TypeDef::new_prim(BOOL_NAME, bool_struct);

        env.reserve_type_id(BOOL_ID, true)?;
        env.register_type(BOOL_NAME, BOOL_ID, bool_type)?;

        Ok(())
    }

    pub fn setup_bool_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let _bool_struct = BOOL_ID.get_from(env).ink();

        self.build_free_ptr_fn(BOOL_ID, primitive_unalloc, env)?;
        self.build_copy_ptr_fn(BOOL_ID, env)?;

        Ok(())
    }
}
