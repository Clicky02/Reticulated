use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyType, BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};

pub struct Environment<'ctx> {
    cur_function: FunctionValue<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    parent: Option<Box<Environment<'ctx>>>,
}

impl<'ctx> Environment<'ctx> {
    pub fn new(cur_function: FunctionValue<'ctx>) -> Self {
        Self {
            cur_function,
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn get_var(&self, ident: &str) -> Option<(PointerValue<'ctx>, BasicTypeEnum<'ctx>)> {
        let mut cur = self;
        loop {
            if self.variables.contains_key(ident) {
                return self.variables.get(ident).cloned();
            }

            if let Some(parent) = &cur.parent {
                cur = &parent;
            } else {
                return None;
            }
        }
    }

    pub fn insert_var(&mut self,ident: String, var_ptr: PointerValue<'ctx>, ptr_type: BasicTypeEnum<'ctx>) {
        self.variables.insert(ident, (var_ptr, ptr_type));
    }
}
