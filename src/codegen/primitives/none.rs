use crate::codegen::{
    env::{id::NONE_ID, Environment},
    err::GenError,
    CodeGen,
};

pub const NONE_NAME: &str = "None";

impl<'ctx> CodeGen<'ctx> {
    pub fn setup_none_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let none_struct = self.create_struct_type(NONE_NAME, vec![]);
        env.reserve_type_id(NONE_ID, NONE_NAME, none_struct)?;

        Ok(())
    }
}
