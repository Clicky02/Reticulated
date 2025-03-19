use inkwell::context::Context;
use inkwell::module::Module as InkwellModule;

pub struct Module<'ctx> {
    inner: InkwellModule<'ctx>,
}

impl<'ctx> Module<'ctx> {
    pub fn new(module: InkwellModule<'ctx>) -> Self {
        Module { inner: module }
    }

    pub fn get_inner(&self) -> &InkwellModule<'ctx> {
        &self.inner
    }

    // Add more wrapper methods as needed
}
