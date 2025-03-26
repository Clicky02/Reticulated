use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};

pub trait InkValueExt<'ctx> {
    fn is_const(self) -> bool;
}

impl<'ctx> InkValueExt<'ctx> for BasicValueEnum<'ctx> {
    fn is_const(self) -> bool {
        match self {
            BasicValueEnum::IntValue(val) => val.is_const(),
            BasicValueEnum::FloatValue(val) => val.is_const(),
            BasicValueEnum::PointerValue(val) => val.is_const(),
            BasicValueEnum::VectorValue(val) => val.is_const(),
            BasicValueEnum::ArrayValue(val) => val.is_const(),
            BasicValueEnum::StructValue(val) => val.is_const(),
        }
    }
}

pub trait InkTypeExt<'ctx> {
    fn get_poison(self) -> BasicValueEnum<'ctx>;
}

impl<'ctx> InkTypeExt<'ctx> for BasicTypeEnum<'ctx> {
    fn get_poison(self) -> BasicValueEnum<'ctx> {
        match self {
            BasicTypeEnum::IntType(ty) => ty.get_poison().into(),
            BasicTypeEnum::FloatType(ty) => ty.get_poison().into(),
            BasicTypeEnum::PointerType(ty) => ty.get_poison().into(),
            BasicTypeEnum::VectorType(ty) => ty.get_poison().into(),
            BasicTypeEnum::ArrayType(ty) => ty.get_poison().into(),
            BasicTypeEnum::StructType(ty) => ty.get_poison().into(),
        }
    }
}
