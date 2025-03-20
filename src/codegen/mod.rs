use env::{
    id::{TypeId, BOOL_ID, FLOAT_ID, INT_ID},
    type_def::TypeDef,
    Environment,
};
use err::GenError;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::StructType,
    values::{BasicValueEnum, PointerValue},
};

pub mod env;
pub mod err;
pub mod primitives;
pub mod to_meta;

use crate::parser::{BinaryOp, Expression, Primary, Statement, UnaryOp};

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

        env.push_scope();

        let fn_type = self.ctx.i64_type().fn_type(&[], false);
        let main_fn = env.module.add_function("main", fn_type, None);
        let entry = self.ctx.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for statement in program {
            self.compile_statement(&statement, &mut env).unwrap();
        }

        env.pop_scope();

        self.builder
            .build_return(Some(&self.ctx.i64_type().const_int(0, false)))
            .unwrap();

        return env.module;
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

                expr_ptr.set_name(identifier);
                env.insert_var(identifier.clone(), expr_ptr, expr_type_id);
            }
            Statement::Assignment {
                identifier,
                expression,
            } => {
                let (var_ptr, var_type_id) = env.get_var(identifier)?;
                let (expr_ptr, expr_type_id) = self.compile_expression(expression, env)?;

                assert_eq!(var_type_id, expr_type_id); // TODO: Gen Error

                let expr_type = env.get_type(expr_type_id);
                let expr_val = self.builder.build_load(expr_type.ink(), expr_ptr, "tmp")?;

                self.builder.build_store(var_ptr, expr_val)?;
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
            } => {
                // let (param_types, is_var_args) = self.params_to_types(parameters, env);
                // let fn_type = self
                //     .ident_to_type(&return_identifier, env)
                //     .fn_type(&param_types, is_var_args);
                // self.module
                //     .add_function(&identifier, fn_type, Some(Linkage::External));
                todo!()
            }
            Statement::Expression(expression) => {
                self.compile_expression(expression, env)?;
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let (cond_ptr, cond_type_id) = self.compile_expression(condition, env)?;
                let cond_type = env.get_type(cond_type_id);
                // TODO: Check boolean?
                let mut cond_val = self.extract_primitive(cond_ptr, cond_type.ink())?;

                let mut source_block = self.builder.get_insert_block().unwrap();
                let func = source_block.get_parent().unwrap();

                let mut then_block = self.ctx.append_basic_block(func, "then");
                let merge_block = self.ctx.append_basic_block(func, "merge");

                // Primary/Then branch
                self.builder.position_at_end(then_block);
                env.push_scope();
                for statement in then_branch {
                    self.compile_statement(statement, env)?;
                }
                env.pop_scope();
                self.builder.build_unconditional_branch(merge_block)?;

                // Else If branches
                for (condition, branch) in else_if_branches {
                    // Create new branches
                    let next_source_block = self
                        .ctx
                        .insert_basic_block_after(then_block, "elseifcondition");
                    let next_then_block = self
                        .ctx
                        .insert_basic_block_after(next_source_block, "elseif");

                    // Create branch from previous block to this new block
                    self.builder.position_at_end(source_block);
                    self.builder.build_conditional_branch(
                        cond_val.into_int_value(),
                        then_block,
                        next_source_block,
                    )?;

                    // Compile condition
                    self.builder.position_at_end(next_source_block);
                    let (cond_ptr, cond_type_id) = self.compile_expression(condition, env)?;
                    let cond_type = env.get_type(cond_type_id);
                    // TODO: Check boolean?

                    cond_val = self.extract_primitive(cond_ptr, cond_type.ink())?;
                    source_block = next_source_block;
                    then_block = next_then_block;

                    // Compile branch statements
                    self.builder.position_at_end(then_block);
                    env.push_scope();
                    for statement in branch {
                        self.compile_statement(statement, env)?;
                    }
                    env.pop_scope();
                    self.builder.build_unconditional_branch(merge_block)?;
                }

                // Else
                if let Some(else_branch) = else_branch {
                    // Create else branch
                    let else_block = self.ctx.insert_basic_block_after(then_block, "else");

                    // Create branch from previous block to this new block
                    self.builder.position_at_end(source_block);
                    self.builder.build_conditional_branch(
                        cond_val.into_int_value(),
                        then_block,
                        else_block,
                    )?;

                    self.builder.position_at_end(else_block);
                    env.push_scope();
                    for statement in else_branch {
                        self.compile_statement(statement, env)?;
                    }
                    env.pop_scope();
                    self.builder.build_unconditional_branch(merge_block)?;
                } else {
                    // Create branch from previous block to outside the if statement
                    self.builder.position_at_end(source_block);
                    self.builder.build_conditional_branch(
                        cond_val.into_int_value(),
                        then_block,
                        merge_block,
                    )?;
                }

                self.builder.position_at_end(merge_block);
            }
            Statement::ReturnStatement { expression: _ } => todo!(),
        };

        Ok(())
    }

    pub fn compile_expression(
        &mut self,
        expression: &Expression,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let val = match expression {
            Expression::Binary(left, op, right) => self.compile_binary(left, op, right, env)?,
            Expression::Unary(op, expr) => self.compile_unary(op, expr, env)?,
            Expression::Invoke(..) => todo!(),
            Expression::Primary(primary) => self.compile_primary(primary, env)?,
        };

        Ok(val)
    }

    // fn compile_invoke(
    //     &mut self,
    //     callee: &Box<Expression>,
    //     args: &Vec<Expression>,
    //     env: &mut env::Environment<'ctx>,
    // ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
    //     let Expression::Primary(Primary::Identifier(ref callee)) = **callee else {
    //         todo!("Add support for first order functions.")
    //     };

    //     let fn_val = env.module().get_function(&callee).unwrap();
    //     let mut arg_values = Vec::new();

    //     for arg in args {
    //         arg_values.push(BasicMetadataValueEnum::StructValue(
    //             self.compile_expression(arg, env)?,
    //         ));
    //     }

    //     Ok(self
    //         .builder
    //         .build_call(fn_val, &arg_values, "calltmp")?
    //         .try_as_basic_value()
    //         .left()
    //         .ok_or(GenError::Call)?
    //         .into_struct_value())
    // }

    fn compile_binary(
        &mut self,
        left: &Box<Expression>,
        op: &BinaryOp,
        right: &Box<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (left_ptr, left_type) = self.compile_expression(left, env)?;
        let (right_ptr, right_type) = self.compile_expression(right, env)?;

        let op_func_name = op.fn_name();
        let op_func = env
            .find_func(op_func_name, Some(left_type), &[left_type, right_type])?
            .get_from(env);

        let ret_type = env.get_type(op_func.ret_type);
        let fn_ret =
            self.builder
                .build_call(op_func.ink(), &[left_ptr.into(), right_ptr.into()], "tmp")?;
        let ret_ptr = self.builder.build_alloca(ret_type.ink(), "tmp")?;
        self.builder.build_store(
            ret_ptr,
            fn_ret
                .try_as_basic_value()
                .unwrap_left()
                .into_struct_value(),
        )?;

        Ok((ret_ptr, op_func.ret_type))
    }

    fn compile_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Box<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_type) = self.compile_expression(expr, env)?;

        let op_fn_name = op.fn_name();
        let op_func = env
            .find_func(op_fn_name, Some(expr_type), &[expr_type])?
            .get_from(env);

        let ret_type = env.get_type(op_func.ret_type);
        let fn_ret = self
            .builder
            .build_call(op_func.ink(), &[expr_ptr.into()], "tmp")?;
        let ret_ptr = self.builder.build_alloca(ret_type.ink(), "tmp")?;
        self.builder.build_store(
            ret_ptr,
            fn_ret
                .try_as_basic_value()
                .unwrap_left()
                .into_struct_value(),
        )?;

        Ok((ret_ptr, op_func.ret_type))
    }

    fn compile_primary(
        &mut self,
        primary: &Primary,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        match primary {
            Primary::Identifier(ident) => Ok(env.get_var(ident)?),
            Primary::Integer(val) => {
                let int_type = env.get_type(INT_ID);
                let inner_int = self.ctx.i64_type().const_int(*val as u64, true);
                let ptr = self.build_struct_pointer(int_type, &[inner_int.into()])?;
                Ok((ptr, INT_ID))
            }
            Primary::Float(val) => {
                let float_type = env.get_type(FLOAT_ID);
                let inner_float = self.ctx.f64_type().const_float(*val);
                let ptr = self.build_struct_pointer(float_type, &[inner_float.into()])?;
                Ok((ptr, FLOAT_ID))
            }
            Primary::String(..) => todo!(),
            Primary::Bool(val) => {
                let bool_type = env.get_type(BOOL_ID);
                let inner_bool = self.ctx.bool_type().const_int((*val) as u64, false);
                let ptr = self.build_struct_pointer(bool_type, &[inner_bool.into()])?;
                Ok((ptr, BOOL_ID))
            }
            Primary::None => todo!(),
            Primary::Grouping(expr) => self.compile_expression(expr, env),
        }
    }

    fn build_struct_pointer(
        &mut self,
        type_def: &TypeDef<'ctx>,
        values: &[BasicValueEnum<'ctx>],
    ) -> Result<PointerValue<'ctx>, GenError> {
        let ink_type: StructType<'ctx> = type_def.ink();

        let val = ink_type.const_named_struct(values);
        let val_ptr = self.builder.build_alloca(ink_type, "literal")?; // TODO: Memory Mangement ???? MALLOC??? RC???
        self.builder.build_store(val_ptr, val)?;

        Ok(val_ptr)
    }
}
