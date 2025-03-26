use inkwell::types::StructType;

use super::id::TypeId;

// #[derive(Debug)]
// pub enum TypeInfo {
//     Primitive,
//     UserDefined { fields: Vec<TypeId> },
// }

#[derive(Debug)]
pub struct TypeDef<'ctx> {
    ident: String,
    value: StructType<'ctx>,
    fields: Vec<TypeId>,
}

impl<'ctx> TypeDef<'ctx> {
    pub fn new(ident: impl ToString, value: StructType<'ctx>, fields: Vec<TypeId>) -> Self {
        Self {
            ident: ident.to_string(),
            value,
            fields,
        }
    }

    pub fn new_prim(ident: impl ToString, value: StructType<'ctx>) -> Self {
        Self {
            ident: ident.to_string(),
            value,
            fields: Vec::new(),
        }
    }

    pub fn ink(&self) -> StructType<'ctx> {
        self.value
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }

    pub fn fields(&self) -> &[TypeId] {
        &self.fields
    }
}

impl<'ctx> Into<StructType<'ctx>> for &TypeDef<'ctx> {
    fn into(self) -> StructType<'ctx> {
        self.ink()
    }
}
