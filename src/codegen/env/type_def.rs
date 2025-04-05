use inkwell::types::StructType;

use crate::codegen::err::GenError;

use super::id::TypeId;

// #[derive(Debug)]
// pub enum TypeInfo {
//     Primitive,
//     UserDefined { fields: Vec<TypeId> },
// }

#[derive(Debug, Clone)]
pub struct Field(u32, String, TypeId); // index, ident, type

impl Field {
    pub fn new(index: u32, ident: impl ToString, tid: TypeId) -> Self {
        Self(index, ident.to_string(), tid)
    }

    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn ident(&self) -> &str {
        &self.1
    }

    pub fn tid(&self) -> TypeId {
        self.2
    }
}

#[derive(Debug)]
pub struct TypeDef<'ctx> {
    ident: String,
    value: StructType<'ctx>,
    fields: Vec<Field>,
}

impl<'ctx> TypeDef<'ctx> {
    pub fn new(ident: impl ToString, value: StructType<'ctx>, fields: Vec<Field>) -> Self {
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

    pub fn find_field(&self, ident: &str) -> Result<&Field, GenError> {
        for field in &self.fields {
            if field.1 == ident {
                return Ok(field);
            }
        }
        Err(GenError::FieldNotFound)
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}

impl<'ctx> Into<StructType<'ctx>> for &TypeDef<'ctx> {
    fn into(self) -> StructType<'ctx> {
        self.ink()
    }
}
