use inkwell::{values::FunctionValue, AddressSpace};

use crate::parser::{Expression, FuncDeclaration};

use super::{
    env::{id::TypeId, Environment},
    err::GenError,
    CodeGen,
};

struct ParamInfo<'a>(&'a str, TypeId, bool);

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn preprocess_fn(
        &mut self,
        owner: Option<TypeId>,
        fn_dec: &FuncDeclaration,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let mut param_types = Vec::new();
        if fn_dec.takes_self {
            if let Some(owner) = owner {
                param_types.push(owner);
            } else {
                return Err(GenError::InvalidFunctionDefinition);
            }
        }

        let mut is_var_args = false;
        for param in &fn_dec.params {
            if is_var_args {
                return Err(GenError::InvalidFunctionDefinition);
            }

            if param.var_args {
                is_var_args = true;
            } else {
                let param_type = env.find_type(&param.type_identifier)?;
                param_types.push(param_type);
            }
        }

        let ret_type = env.find_type(&fn_dec.return_identifier)?;
        env.create_func(
            owner,
            &fn_dec.identifier,
            &param_types,
            ret_type,
            is_var_args,
        )?;

        Ok(()) // TODO: Make a preprocessed statement enum?
    }

    pub(super) fn compile_fn(
        &mut self,
        owner: Option<TypeId>,
        fn_dec: &FuncDeclaration,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let mut param_info = fn_dec
            .params
            .iter()
            .map(|p| {
                Ok(ParamInfo(
                    &p.identifier,
                    env.find_type(&p.type_identifier)?,
                    p.var_args,
                ))
            })
            .collect::<Result<Vec<_>, GenError>>()?;

        if let Some(owner) = owner {
            if fn_dec.takes_self {
                param_info.insert(0, ParamInfo("self", owner, false));
            } else {
                return Err(GenError::InvalidFunctionDefinition);
            }
        }

        let fn_id = env.find_func(
            &fn_dec.identifier,
            owner,
            &param_info.iter().map(|info| info.1).collect::<Vec<_>>(),
        )?;
        let fn_val = env.get_func(fn_id).ink();

        let prev_block = self.builder.get_insert_block().unwrap();
        let entry_block = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_block);

        let containing_fn_env = env.new_fn_env(fn_id, false);

        env.push_scope();
        self.create_fn_variables(fn_val, &param_info, env)?;
        self.compile_block(&fn_dec.body, env)?;
        env.pop_scope();

        env.set_fn_env(containing_fn_env);

        self.builder.position_at_end(prev_block);

        Ok(())
    }

    pub(super) fn compile_return(
        &mut self,
        expr: &Expression,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (expr_ptr, expr_tid) = self.compile_expression(expr, env)?;
        let fn_def = env.get_cur_fn();

        if fn_def.ret_type != expr_tid {
            return Err(GenError::InvalidType);
        }

        env.func.scopes.last_mut().unwrap().set_returned();
        for scope in env.func.scopes.iter().rev() {
            self.free_vars_in_scope(scope, env)?
        }

        self.builder.build_return(Some(&expr_ptr))?;

        Ok(())
    }

    fn create_fn_variables(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        params: &Vec<ParamInfo<'_>>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let ptr_type = self.ctx.ptr_type(AddressSpace::default());
        Ok(for i in 0..params.len() {
            let param = &params[i];
            if !param.2 {
                let param_var = self
                    .builder
                    .build_alloca(ptr_type, &(param.0.to_owned() + "_var"))?;

                self.builder.build_store(
                    param_var,
                    fn_val.get_nth_param(i as u32).unwrap().into_pointer_value(),
                )?;

                env.insert_var(param.0.to_string(), param_var, param.1);
            } else {
                todo!("Figure out how to implement variable length parameters")
            }
        })
    }
}
