use std::collections::HashMap;

use inkwell::values::PointerValue;

use super::id::{FunctionId, TypeId};

#[derive(Debug)]
pub struct FuncEnvironment<'ctx> {
    pub fn_id: FunctionId,
    pub is_script: bool,
    pub scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> FuncEnvironment<'ctx> {
    pub fn new(fn_id: FunctionId, is_script: bool) -> Self {
        Self {
            fn_id,
            is_script,
            scopes: vec![],
        }
    }
}

#[derive(Default, Debug)]
pub struct Scope<'ctx> {
    pub(super) variables: HashMap<String, (PointerValue<'ctx>, TypeId)>,
    pub has_returned: bool,
}

impl<'ctx> Scope<'ctx> {
    pub fn variables(&self) -> &HashMap<String, (PointerValue<'ctx>, TypeId)> {
        &self.variables
    }

    pub fn set_returned(&mut self) {
        self.has_returned = true;
    }
}
