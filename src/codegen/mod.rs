use env::{id::INT_ID, Environment};
use err::GenError;
use inkwell::{builder::Builder, context::Context, module::Module, AddressSpace};

pub mod builtin;
pub mod control;
pub mod env;
pub mod err;
pub mod expr;
pub mod func;
pub mod ink_extension;
pub mod structs;
pub mod util;

use crate::parser::{Primary, Statement};

// TODO: Not pub
pub struct CodeGen<'ctx> {
    pub ctx: &'ctx Context,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        Self {
            ctx,
            builder: ctx.create_builder(),
        }
    }

    pub fn gen_code_for(&mut self, program: Vec<Statement>) -> Module<'ctx> {
        let module = self.ctx.create_module("main");

        // Declare main function
        let fn_type = self.ctx.i64_type().fn_type(&[], false);
        let main_fn = module.add_function("main", fn_type, None);
        let main_entry = self.ctx.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_entry);

        // Setup environment
        let mut env = Environment::new(module);
        self.setup_builtins(&mut env).unwrap();

        // Setup and compile top-level code
        let (script_fn_val, script_fn_id) = env
            .create_func(None, "$script", &[], INT_ID, false)
            .unwrap();
        let script_entry = self.ctx.append_basic_block(script_fn_val, "entry");
        self.builder.position_at_end(script_entry);

        env.new_fn_env(script_fn_id, true);
        self.compile_block(&program, &mut env).unwrap();

        // Create main function
        self.builder.position_at_end(main_entry);
        let script_result = self
            .builder
            .build_direct_call(script_fn_val, &[], "result")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        let ret_val = self
            .extract_primitive(script_result, env.get_type(INT_ID).ink())
            .unwrap();

        self.free_pointer(script_result, INT_ID, &env).unwrap();

        self.builder.build_return(Some(&ret_val)).unwrap();

        return env.module;
    }

    pub fn compile_block(
        &mut self,
        statements: &[Statement],
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        env.push_scope();
        for statement in statements {
            self.preprocess_statement(&statement, env).unwrap();
        }

        for statement in statements {
            self.compile_statement(&statement, env).unwrap();
        }

        let prev_scope = env.pop_scope().unwrap();

        if !prev_scope.has_returned {
            self.free_vars_in_scope(&prev_scope, env)?;

            // Make sure every script always returns
            if env.func.scopes.len() == 0 && env.func.is_script {
                let (ret_val, ..) = self.compile_primary(&Primary::Integer(0), env)?;
                self.builder.build_return(Some(&ret_val))?;
            }
        }

        Ok(())
    }

    pub fn preprocess_statement(
        &mut self,
        statement: &Statement,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(), GenError> {
        match statement {
            Statement::FunctionDeclaration {
                identifier,
                parameters,
                return_identifier,
                body,
            } => self.preprocess_fn(identifier, parameters, return_identifier, body, env),
            Statement::StructDefinition { identifier, fields } => {
                self.preprocess_struct_definition(identifier, fields, env)
            }
            _ => Ok(()),
        }
    }

    pub fn compile_statement(
        &mut self,
        statement: &Statement,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(), GenError> {
        match statement {
            Statement::Declaration {
                identifier,
                type_identifier,
                expression,
            } => {
                let var_type_id = env.find_type(type_identifier)?;
                let (expr_ptr, expr_type_id) = self.compile_expression(expression, env)?;

                assert_eq!(var_type_id, expr_type_id); // TODO: Gen Error

                // TODO: Do variables need to be reference counted?
                let ptr_type = self.ctx.ptr_type(AddressSpace::default());
                let var_ptr = self.builder.build_alloca(ptr_type, identifier)?;
                self.builder.build_store(var_ptr, expr_ptr)?; // Store the expression pointer in the variable.

                env.insert_var(identifier.clone(), var_ptr, expr_type_id);
            }
            Statement::Assignment {
                identifier,
                expression,
            } => {
                let (var_ptr, var_type_id) = env.get_var(identifier)?;
                let (expr_ptr, expr_type_id) = self.compile_expression(expression, env)?;

                let old_expr_ptr = self.builder.build_load(
                    self.ctx.ptr_type(AddressSpace::default()),
                    var_ptr,
                    "prev_expr_ptr",
                )?;
                self.free_pointer(old_expr_ptr.into_pointer_value(), var_type_id, env)?;

                assert_eq!(var_type_id, expr_type_id); // TODO: Gen Error

                self.builder.build_store(var_ptr, expr_ptr)?;
            }
            Statement::FunctionDeclaration {
                identifier,
                parameters,
                return_identifier,
                body,
            } => {
                self.compile_fn(identifier, parameters, return_identifier, body, env)?;
            }
            Statement::ExternFunctionDeclaration {
                identifier: _,
                parameters: _,
                return_identifier: _,
            } => todo!(),
            Statement::ReturnStatement { expression: expr } => {
                self.compile_return(expr, env)?;
            }
            Statement::Expression(expression) => {
                let (ptr, ptr_type_id) = self.compile_expression(expression, env)?;
                self.free_pointer(ptr, ptr_type_id, env)?;
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => self.compile_if_statement(
                condition,
                then_branch,
                else_if_branches,
                else_branch,
                env,
            )?,
            Statement::StructDefinition { identifier, fields } => {
                self.compile_struct_definition(identifier, fields, env)?;
            }
            Statement::WhileLoop { condition, block } => {
                self.compile_while_loop(condition, block, env)?
            }
        };

        Ok(())
    }
}
