use inkwell::{values::FunctionValue, AddressSpace};

use crate::parser::{Expression, FuncParameter, Statement};

use super::{
    env::{id::TypeId, Environment},
    err::GenError,
    CodeGen,
};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn preprocess_fn(
        &mut self,
        ident: &str,
        params: &Vec<FuncParameter>,
        ret_ident: &str,
        _body: &Vec<Statement>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let mut param_types = Vec::new();
        let mut is_var_args = false;
        for param in params {
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

        let ret_type = env.find_type(ret_ident)?;
        env.create_func(None, ident, &param_types, ret_type, is_var_args)?;

        Ok(()) // TODO: Make a preprocessed statement enum?
    }

    pub(super) fn compile_fn(
        &mut self,
        ident: &str,
        params: &Vec<FuncParameter>,
        _ret_ident: &str,
        body: &Vec<Statement>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let param_types = params
            .iter()
            .filter(|p| !p.var_args)
            .map(|p| env.find_type(&p.type_identifier))
            .collect::<Result<Vec<_>, GenError>>()?;

        let fn_id = env.find_func(ident, None, &param_types)?;
        let fn_val = env.get_func(fn_id).ink();

        let prev_block = self.builder.get_insert_block().unwrap();
        let entry_block = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_block);

        let containing_fn_env = env.new_fn_env(fn_id, false);

        env.push_scope();
        self.create_fn_variables(fn_val, params, param_types, env)?;
        self.compile_block(body, env)?;
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
        params: &Vec<FuncParameter>,
        param_types: Vec<TypeId>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let ptr_type = self.ctx.ptr_type(AddressSpace::default());
        Ok(for i in 0..params.len() {
            let param = &params[i];
            if !param.var_args {
                let param_var = self
                    .builder
                    .build_alloca(ptr_type, &(param.identifier.to_owned() + "_var"))?;

                self.builder.build_store(
                    param_var,
                    fn_val.get_nth_param(i as u32).unwrap().into_pointer_value(),
                )?;

                env.insert_var(param.identifier.to_string(), param_var, param_types[i]);
            } else {
                todo!("Figure out how to implement variable length parameters")
            }
        })
    }
}
