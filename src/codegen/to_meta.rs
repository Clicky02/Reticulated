use inkwell::{types::{BasicMetadataTypeEnum, BasicTypeEnum}, values::{BasicMetadataValueEnum, BasicValueEnum}};

pub trait ToMetaTypeEnum<'ctx> {
    fn to_meta_type(self) -> BasicMetadataTypeEnum<'ctx>;
}

impl<'ctx> ToMetaTypeEnum<'ctx> for BasicTypeEnum<'ctx> {
    fn to_meta_type(self) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            BasicTypeEnum::ArrayType(array_type) => BasicMetadataTypeEnum::ArrayType(array_type),
            BasicTypeEnum::FloatType(float_type) => BasicMetadataTypeEnum::FloatType(float_type),
            BasicTypeEnum::IntType(int_type) => BasicMetadataTypeEnum::IntType(int_type),
            BasicTypeEnum::PointerType(pointer_type) => BasicMetadataTypeEnum::PointerType(pointer_type),
            BasicTypeEnum::StructType(struct_type) =>   BasicMetadataTypeEnum::StructType(struct_type),
            BasicTypeEnum::VectorType(vector_type) => BasicMetadataTypeEnum::VectorType(vector_type),
        }
    }
}

pub trait ToMetaValueEnum<'ctx> {
    fn to_meta_val(self) -> BasicMetadataValueEnum<'ctx>;
}

impl<'ctx> ToMetaValueEnum<'ctx> for BasicValueEnum<'ctx> {
    fn to_meta_val(self) -> BasicMetadataValueEnum<'ctx> {
        match self {
            BasicValueEnum::ArrayValue(array_value) => BasicMetadataValueEnum::ArrayValue(array_value),
            BasicValueEnum::IntValue(int_value) => BasicMetadataValueEnum::IntValue(int_value),
            BasicValueEnum::FloatValue(float_value) => BasicMetadataValueEnum::FloatValue(float_value),
            BasicValueEnum::PointerValue(pointer_value) => BasicMetadataValueEnum::PointerValue(pointer_value),
            BasicValueEnum::StructValue(struct_value) => BasicMetadataValueEnum::StructValue(struct_value),
            BasicValueEnum::VectorValue(vector_value) => BasicMetadataValueEnum::VectorValue(vector_value),
        }
    }
}