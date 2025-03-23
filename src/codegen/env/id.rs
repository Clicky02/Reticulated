use super::{fn_def::FuncDef, type_def::TypeDef, Environment};

pub const NONE_ID: TypeId = TypeId(0);
pub const INT_ID: TypeId = TypeId(1);
pub const FLOAT_ID: TypeId = TypeId(2);
pub const BOOL_ID: TypeId = TypeId(3);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct TypeId(pub(super) u64);

impl TypeId {
    pub fn get_from<'ctx, 'env>(&self, env: &'env Environment<'ctx>) -> &'env TypeDef<'ctx> {
        env.get_type(*self)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct FunctionId(pub(super) u64, pub(super) u64);

impl FunctionId {
    pub fn get_from<'ctx, 'env>(&self, env: &'env Environment<'ctx>) -> &'env FuncDef<'ctx> {
        env.get_func(*self)
    }
}
