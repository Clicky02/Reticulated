use inkwell::types;

use super::{RetType, TypeEnum};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionType<'ctx> {
    value: types::FunctionType<'ctx>,
    param_types: Vec<TypeEnum<'ctx>>,
    return_type: Box<TypeEnum<'ctx>>,
}

impl<'ctx> FunctionType<'ctx> {
    pub(crate) fn new(
        value: types::FunctionType<'ctx>,
        param_types: Vec<TypeEnum<'ctx>>,
        return_type: TypeEnum<'ctx>,
    ) -> Self {
        Self {
            value,
            param_types,
            return_type: Box::new(return_type),
        }
    }
}

impl<'ctx> RetType<'ctx> for FunctionType<'ctx> {
    fn as_type_enum(&self) -> TypeEnum<'ctx> {
        TypeEnum::FunctionType(self.clone())
    }

    fn try_as_inkwell_basic(&self) -> types::BasicTypeEnum<'ctx> {
        self.value.
    }
}
