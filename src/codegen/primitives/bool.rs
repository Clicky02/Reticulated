use crate::codegen::{
    env::{id::BOOL_ID, Environment},
    err::GenError,
    CodeGen,
};

pub const BOOL_NAME: &str = "bool";

impl<'ctx> CodeGen<'ctx> {
    pub fn setup_bool_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let bool_struct = self.create_struct_type(BOOL_NAME, vec![self.ctx.bool_type().into()]);
        env.reserve_type_id(BOOL_ID, BOOL_NAME, bool_struct)?;

        Ok(())
    }
}
