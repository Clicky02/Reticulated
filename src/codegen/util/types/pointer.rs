use inkwell::types::{self, BasicType};

use super::{RetType, TypeEnum};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PointerType<'ctx> {
    value: types::PointerType<'ctx>,
    ptd_type: Box<TypeEnum<'ctx>>,
}

impl<'ctx> PointerType<'ctx> {
    pub(crate) fn new(value: types::PointerType<'ctx>, ptd_type: TypeEnum<'ctx>) -> Self {
        Self {
            value,
            ptd_type: Box::new(ptd_type),
        }
    }
}

impl<'ctx> RetType<'ctx> for PointerType<'ctx> {
    fn as_type_enum(&self) -> TypeEnum<'ctx> {
        TypeEnum::PointerType(self.clone())
    }

    fn try_as_inkwell_basic(&self) -> Result<types::BasicTypeEnum<'ctx>, String> {
        Ok(self.value.as_basic_type_enum())
    }
}
