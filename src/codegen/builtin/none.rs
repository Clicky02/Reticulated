use inkwell::{values::PointerValue, AddressSpace};

use crate::codegen::{
    env::{id::NONE_ID, type_def::TypeDef, Environment},
    err::GenError,
    CodeGen,
};

pub const NONE_NAME: &str = "None";
pub const NONE_CONST: &str = "const_none";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_none_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let none_struct = self.create_struct_type(NONE_NAME, vec![]);
        let none_type = TypeDef::new_prim(NONE_NAME, none_struct);

        env.reserve_type_id(NONE_ID, true)?;
        env.register_type(NONE_NAME, NONE_ID, none_type)?;

        Ok(())
    }

    pub fn setup_none_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        self.build_noop_copy_ptr_fn(NONE_ID, env)?;
        self.build_noop_free_ptr_fn(NONE_ID, env)?;

        Ok(())
    }

    pub fn build_none(
        &mut self,
        _env: &mut Environment<'ctx>,
    ) -> Result<PointerValue<'ctx>, GenError> {
        // TODO: Real None values?
        Ok(self.ctx.ptr_type(AddressSpace::default()).const_null())
    }
}
