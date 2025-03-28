use inkwell::values::FunctionValue;

use super::id::TypeId;

pub fn create_fn_name(
    fn_ident: &str,
    type_ident: Option<&str>,
    param_type_idents: &[&str],
) -> String {
    let mut name = String::with_capacity(fn_ident.len() * (2 + param_type_idents.len()) + 10); // Rough approximation of string length

    if let Some(type_ident) = type_ident {
        name.push_str(type_ident);
        name.push_str(".");
    }

    name.push_str(fn_ident);
    name.push_str("-");

    for type_name in param_type_idents {
        name.push_str(&type_name);
        name.push_str(".");
    }

    name
}

#[derive(Debug)]
pub struct FuncDef<'ctx> {
    pub ident: String,
    pub value: FunctionValue<'ctx>,
    pub params: Vec<TypeId>,
    pub ret_type: TypeId,
}

impl<'ctx> FuncDef<'ctx> {
    pub fn new(
        ident: impl ToString,
        value: FunctionValue<'ctx>,
        params: Vec<TypeId>,
        ret_type: TypeId,
    ) -> Self {
        Self {
            ident: ident.to_string(),
            value,
            params,
            ret_type,
        }
    }

    pub fn ink(&self) -> FunctionValue<'ctx> {
        self.value
    }
}
