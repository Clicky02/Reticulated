use std::collections::HashMap;

use fn_def::{create_fn_name, FuncDef};
use id::{FunctionId, TypeId};
use inkwell::{
    module::Module,
    types::{BasicMetadataTypeEnum, StructType},
    values::{FunctionValue, PointerValue},
    AddressSpace,
};
use scope::Scope;
use type_def::TypeDef;

use super::err::GenError;

pub mod fn_def;
pub mod id;
pub mod scope;
pub mod type_def;

#[derive(Debug)]
pub struct Environment<'ctx> {
    pub module: Module<'ctx>,
    pub scopes: Vec<Scope<'ctx>>,

    next_type_id: u64,
    type_ids: HashMap<String, TypeId>,
    types: HashMap<TypeId, TypeDef<'ctx>>,

    next_fn_id: u64,
    fn_ids: HashMap<String, FunctionId>,
    fns: HashMap<FunctionId, FuncDef<'ctx>>,
}

impl<'ctx> Environment<'ctx> {
    pub fn new(module: Module<'ctx>) -> Self {
        Self {
            module,
            scopes: vec![],

            next_type_id: 1,
            type_ids: HashMap::new(),
            types: HashMap::new(),

            next_fn_id: 1,
            fn_ids: HashMap::new(),
            fns: HashMap::new(),
        }
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn get_var(&self, ident: &str) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        for scope in self.scopes.iter().rev() {
            if scope.variables.contains_key(ident) {
                return Ok(scope.variables.get(ident).cloned().unwrap());
            }
        }

        Err(GenError::VariableNotFound)
    }

    pub fn insert_var(&mut self, ident: String, var_ptr: PointerValue<'ctx>, ptr_type: TypeId) {
        self.scopes
            .last_mut()
            .unwrap()
            .variables
            .insert(ident, (var_ptr, ptr_type));
    }

    pub fn update_var(&mut self, ident: &str, var_ptr: PointerValue<'ctx>) -> Result<(), GenError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((.., type_id)) = scope.variables.get(ident) {
                scope
                    .variables
                    .insert(ident.to_string(), (var_ptr, *type_id));
                return Ok(());
            }
        }

        Err(GenError::VariableNotFound)
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_type(&self, id: TypeId) -> &TypeDef<'ctx> {
        self.types.get(&id).unwrap()
    }

    pub fn get_type_mut(&mut self, id: TypeId) -> &mut TypeDef<'ctx> {
        self.types.get_mut(&id).unwrap()
    }

    pub fn find_type(&self, ident: &str) -> Result<TypeId, GenError> {
        dbg!(ident);
        self.type_ids
            .get(ident)
            .copied()
            .ok_or(GenError::TypeNotFound)
    }

    // pub fn create_type(
    //     &mut self,
    //     ident: &str,
    //     struct_type: StructType<'ctx>,
    // ) -> Result<TypeId, GenError> {
    //     let type_id = self.gen_type_id();
    //     self.register_type(ident, type_id, TypeDef::new(ident, struct_type));

    //     Ok(type_id)
    // }

    pub fn find_func(
        &self,
        ident: &str,
        owner: Option<TypeId>,
        param_types: &[TypeId],
    ) -> Result<FunctionId, GenError> {
        let fn_name = self.create_fn_name(ident, owner, param_types);

        self.fn_ids
            .get(&fn_name)
            .copied()
            .ok_or(GenError::FunctionNotFound)
    }

    pub fn get_func(&self, id: FunctionId) -> &FuncDef<'ctx> {
        self.fns.get(&id).unwrap()
    }

    pub fn create_func(
        &mut self,
        owner: Option<TypeId>,
        ident: &str,
        param_types: &[TypeId],
        ret_type: Option<TypeId>,
        is_var_args: bool,
    ) -> Result<FunctionValue<'ctx>, GenError> {
        // Ensure the owner type is the first parameter
        if let Some(owner) = owner {
            if param_types.len() == 0 || param_types[0] != owner {
                return Err(GenError::InvalidFunctionDefinition);
            }
        }

        let id = self.gen_fn_id(owner);
        let fn_name = self.create_fn_name(ident, owner, &param_types);

        let ink_param_types = param_types
            .iter()
            .map(|id| self.types.get(id).unwrap().ink())
            .map(|t| {
                BasicMetadataTypeEnum::PointerType(
                    t.get_context().ptr_type(AddressSpace::default()),
                )
            })
            .collect::<Vec<_>>();

        let ink_fn_type = if let Some(_) = ret_type {
            self.module
                .get_context()
                .ptr_type(AddressSpace::default())
                .fn_type(&ink_param_types, is_var_args)
        } else {
            self.module
                .get_context()
                .void_type()
                .fn_type(&ink_param_types, is_var_args)
        };

        let fn_value = self.module.add_function(&fn_name, ink_fn_type, None);

        self.register_fn(
            &fn_name,
            id,
            FuncDef::new(&fn_name, fn_value, param_types.to_vec(), ret_type),
        );

        Ok(fn_value)
    }

    pub fn type_id_ident(&self, id: TypeId) -> &str {
        self.types.get(&id).unwrap().ident()
    }

    pub fn gen_type_id(&mut self) -> TypeId {
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        id
    }

    pub fn reserve_type_id(&mut self, type_id: TypeId, force: bool) -> Result<(), GenError> {
        if !force && self.next_type_id > type_id.0 {
            return Err(GenError::IdentConflict);
        }

        self.next_type_id = type_id.0 + 1;

        Ok(())
    }

    fn create_fn_name(&self, fn_ident: &str, owner: Option<TypeId>, params: &[TypeId]) -> String {
        create_fn_name(
            fn_ident,
            owner.map(|id| self.type_id_ident(id)),
            &params
                .iter()
                .map(|id| self.type_id_ident(*id))
                .collect::<Vec<_>>(),
        )
    }

    pub fn register_type(
        &mut self,
        ident: &str,
        id: TypeId,
        type_def: TypeDef<'ctx>,
    ) -> Result<(), GenError> {
        if self.type_ids.contains_key(ident) {
            return Err(GenError::IdentConflict);
        }

        self.type_ids.insert(ident.to_string(), id);
        self.types.insert(id, type_def);
        Ok(())
    }

    pub fn gen_fn_id(&mut self, owner: Option<TypeId>) -> FunctionId {
        let id = FunctionId(self.next_fn_id, owner.unwrap_or(TypeId(0)).0);
        self.next_fn_id += 1;
        id
    }

    pub fn register_fn(&mut self, ident: &str, id: FunctionId, fn_def: FuncDef<'ctx>) {
        self.fn_ids.insert(ident.to_string(), id);
        self.fns.insert(id, fn_def);
    }
}
