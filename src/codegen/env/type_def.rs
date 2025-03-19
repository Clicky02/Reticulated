use inkwell::types::StructType;

#[derive(Debug)]
pub struct TypeDef<'ctx> {
    ident: String,
    pub value: StructType<'ctx>,
}

impl<'ctx> TypeDef<'ctx> {
    pub fn new(ident: impl ToString, value: StructType<'ctx>) -> Self {
        Self {
            ident: ident.to_string(),
            value,
        }
    }

    pub fn ink(&self) -> StructType<'ctx> {
        self.value
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }
}

impl<'ctx> Into<StructType<'ctx>> for &TypeDef<'ctx> {
    fn into(self) -> StructType<'ctx> {
        self.ink()
    }
}
