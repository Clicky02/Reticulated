use std::ops::Deref;

use inkwell::{context, AddressSpace};

use super::{builder::Builder, module::Module, types::*};

pub struct Context(context::Context);

// impl Deref for Context {
//     type Target = context::Context;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl Context {
    pub fn create_builder(&self) -> Builder {
        Builder::new(self.0.create_builder())
    }

    pub fn create_module(&self, name: &str) -> Module {
        Module::new(self.0.create_module(name))
    }

    pub fn int_type(&mut self) -> IntType<'_> {
        IntType::new(self.0.i64_type())
    }

    pub fn float_type(&mut self) -> FloatType<'_> {
        FloatType::new(self.0.f64_type())
    }

    pub fn bool_type(&mut self) -> BoolType<'_> {
        BoolType::new(self.0.bool_type())
    }

    // pub fn string_type(&mut self) -> StringType<'_> {
    //     StringType::new(self.0.ptr_type(AddressSpace::default()))
    // }

    pub fn ptr_type<'ctx>(&'ctx mut self, ptd_type: TypeEnum<'ctx>) -> PointerType<'ctx> {
        PointerType::new(self.0.ptr_type(AddressSpace::default()), ptd_type)
    }
}
