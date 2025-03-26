use crate::codegen::{
    env::{id::NONE_ID, type_def::TypeDef, Environment},
    err::GenError,
    CodeGen,
};

pub const NONE_NAME: &str = "None";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_none_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let none_struct = self.create_struct_type(NONE_NAME, vec![]);
        let none_type = TypeDef::new_prim(NONE_NAME, none_struct);

        env.reserve_type_id(NONE_ID, true)?;
        env.register_type(NONE_NAME, NONE_ID, none_type)?;

        Ok(())
    }

    pub fn setup_none_primitive(&mut self, _env: &mut Environment<'ctx>) -> Result<(), GenError> {
        Ok(())
    }
}
