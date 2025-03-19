use std::collections::HashMap;

use inkwell::values::PointerValue;

use super::id::TypeId;

#[derive(Default, Debug)]
pub struct Scope<'ctx> {
    pub(super) variables: HashMap<String, (PointerValue<'ctx>, TypeId)>,
}
