use env::Environment;
use err::GenError;
use inkwell::{builder::Builder, context::Context, module::Module, AddressSpace};

pub mod control;
pub mod env;
pub mod err;
pub mod expr;
pub mod ink_extension;
pub mod primitives;
pub mod structs;
pub mod util;

use crate::parser::Statement;

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

        let mut env = Environment::new(module);
        self.setup_primitive_types(&mut env).unwrap();

        let fn_type = self.ctx.i64_type().fn_type(&[], false);
        let main_fn = env.module.add_function("main", fn_type, None);
        let entry = self.ctx.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        self.compile_block(&program, &mut env).unwrap();

        self.builder
            .build_return(Some(&self.ctx.i64_type().const_int(0, false)))
            .unwrap();

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
        env.pop_scope();

        Ok(())
    }

    pub fn preprocess_statement(
        &mut self,
        statement: &Statement,
        _env: &mut env::Environment<'ctx>,
    ) -> Result<(), GenError> {
        match statement {
            Statement::FunctionDeclaration { .. } => todo!(),
            Statement::StructDefinition { .. } => todo!(),
            _ => (),
        };

        Ok(())
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
                self.destroy_pointer(old_expr_ptr.into_pointer_value(), var_type_id, env)?;

                assert_eq!(var_type_id, expr_type_id); // TODO: Gen Error

                self.builder.build_store(var_ptr, expr_ptr)?;
            }
            Statement::FunctionDeclaration {
                identifier: _,
                parameters: _,
                return_identifier: _,
                body: _,
            } => todo!(),
            Statement::ExternFunctionDeclaration {
                identifier: _,
                parameters: _,
                return_identifier: _,
            } => todo!(),
            Statement::ReturnStatement { expression: expr } => {
                let (expr_ptr, expr_type_id) = self.compile_expression(expr, env)?;
                let expr_type = env.get_type(expr_type_id);
                let mut expr_val = self.builder.build_load(expr_type.ink(), expr_ptr, "tmp")?;

                // At main function
                if env.scopes.len() == 1 {
                    expr_val = self.extract_primitive(expr_ptr, expr_type.ink())?;
                }

                self.builder.build_return(Some(&expr_val))?;
            }
            Statement::Expression(expression) => {
                let (ptr, ptr_type_id) = self.compile_expression(expression, env)?;
                self.destroy_pointer(ptr, ptr_type_id, env)?;
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
            Statement::StructDefinition {
                identifier: _,
                fields: _,
            } => todo!(),
            Statement::WhileLoop { condition, block } => {
                self.compile_while_loop(condition, block, env)?
            }
        };

        Ok(())
    }
}
