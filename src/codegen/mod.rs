use std::ops::Deref;

use env::{
    id::{FunctionId, TypeId, BOOL_ID, FLOAT_ID, INT_ID},
    Environment,
};
use err::GenError;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicMetadataValueEnum, BasicValueEnum, InstructionValue, PointerValue},
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

                self.destroy_pointer(var_ptr, var_type_id, env)?;

                assert_eq!(var_type_id, expr_type_id); // TODO: Gen Error

                env.update_var(identifier, expr_ptr)?;
            }
            Statement::FunctionDeclaration {
                identifier,
                parameters,
                return_identifier,
                body,
            } => todo!(),
            Statement::ExternFunctionDeclaration {
                identifier: _,
                parameters: _,
                return_identifier: _,
            } => todo!(),
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
            Statement::StructDefinition { identifier, fields } => todo!(),
        };

        Ok(())
    }

    fn compile_if_statement(
        &mut self,
        condition: &Expression,
        then_branch: &Vec<Statement>,
        else_if_branches: &Vec<(Expression, Vec<Statement>)>,
        else_branch: &Option<Vec<Statement>>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        // TODO: Decrement Condition Expressions

        let (cond_ptr, cond_type_id) = self.compile_expression(condition, env)?;
        let cond_type = env.get_type(cond_type_id);

        assert_eq!(cond_type_id, BOOL_ID); // TODO: GenError

        let mut cond_val = self.extract_primitive(cond_ptr, cond_type.ink())?;
        self.destroy_pointer(cond_ptr, cond_type_id, env)?;

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
            self.destroy_pointer(cond_ptr, cond_type_id, env)?;

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
            Expression::Invoke(expr, params) => self.compile_invoke(expr, params, env)?,
            Expression::Primary(primary) => self.compile_primary(primary, env)?,
        };

        Ok(val)
    }

    fn compile_invoke(
        &mut self,
        callee: &Box<Expression>,
        args: &Vec<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let Expression::Primary(Primary::Identifier(ident)) = callee.deref() else {
            todo!("Add first-class function support")
        };

        let params = args
            .into_iter()
            .map(|val| self.compile_expression(val, env))
            .collect::<Result<Vec<_>, GenError>>()?;
        let (param_vals, param_types): (Vec<_>, Vec<_>) = params.into_iter().unzip();

        let fn_id = env.find_func(ident, None, &param_types)?;

        self.call_func(fn_id, &param_vals, env)
    }

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
        let op_func_id = env.find_func(op_func_name, Some(left_type), &[left_type, right_type])?;

        let ret = self.call_func(op_func_id, &[left_ptr, right_ptr], env)?;

        self.destroy_pointer(left_ptr, left_type, env)?;
        self.destroy_pointer(right_ptr, right_type, env)?;

        Ok(ret)
    }

    fn compile_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Box<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_type) = self.compile_expression(expr, env)?;

        let op_func_id = env.find_func(op.fn_name(), Some(expr_type), &[expr_type])?;

        let ret = self.call_func(op_func_id, &[expr_ptr], env)?;

        self.destroy_pointer(expr_ptr, expr_type, env)?;

        Ok(ret)
    }

    fn compile_primary(
        &mut self,
        primary: &Primary,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        match primary {
            Primary::Identifier(ident) => {
                let (ptr, type_id) = env.get_var(ident)?;
                let ptr = self.copy_pointer(ptr, type_id, env)?;
                Ok((ptr, type_id))
            }
            Primary::Integer(val) => {
                let int_type = env.get_type(INT_ID);
                let inner_int = self.ctx.i64_type().const_int(*val as u64, true);
                let ptr = self.build_struct(int_type.ink(), vec![inner_int.into()])?;
                Ok((ptr, INT_ID))
            }
            Primary::Float(val) => {
                let float_type = env.get_type(FLOAT_ID);
                let inner_float = self.ctx.f64_type().const_float(*val);
                let ptr = self.build_struct(float_type.ink(), vec![inner_float.into()])?;
                Ok((ptr, FLOAT_ID))
            }
            Primary::String(..) => todo!(),
            Primary::Bool(val) => {
                let bool_type = env.get_type(BOOL_ID);
                let inner_bool = self.ctx.bool_type().const_int((*val) as u64, false);
                let ptr = self.build_struct(bool_type.ink(), vec![inner_bool.into()])?;
                Ok((ptr, BOOL_ID))
            }
            Primary::None => todo!(),
            Primary::Grouping(expr) => self.compile_expression(expr, env),
        }
    }

    // TODO: Could make a more optimized version of this for constants that uses const_named_struct
    fn build_struct(
        &mut self,
        struct_type: StructType<'ctx>,
        mut values: Vec<BasicValueEnum<'ctx>>,
    ) -> Result<PointerValue<'ctx>, GenError> {
        values.push(self.ctx.i64_type().const_int(1, false).into());
        assert_eq!(struct_type.count_fields() as usize, values.len());

        let val_ptr = self.builder.build_malloc(struct_type, "literal")?; // TODO: Memory Mangement ???? MALLOC??? RC???

        for i in 0..struct_type.count_fields() {
            let struct_val_ptr =
                self.builder
                    .build_struct_gep(struct_type, val_ptr, i, "struct_val_ptr")?;
            self.builder
                .build_store(struct_val_ptr, values[i as usize])?;
        }
        Ok(val_ptr)
    }

    fn create_type(
        &mut self,
        ident: &str,
        fields: &[TypeId],
        env: &mut Environment<'ctx>,
    ) -> Result<TypeId, GenError> {
        let field_types = fields
            .iter()
            .map(|id| env.get_type(*id).ink().as_basic_type_enum())
            .collect();

        let struct_type = self.create_struct_type(ident, field_types);
        env.create_type(ident, struct_type)
    }

    fn create_struct_type(
        &self,
        ident: &str,
        mut fields: Vec<BasicTypeEnum<'ctx>>,
    ) -> StructType<'ctx> {
        fields.push(self.ctx.i64_type().into());

        let struct_type = self.ctx.opaque_struct_type(ident);
        struct_type.set_body(&fields, false);
        struct_type
    }

    fn call_func(
        &mut self,
        fn_id: FunctionId,
        args: &[PointerValue<'ctx>],
        env: &mut env::Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let param_vals: Vec<_> = args
            .into_iter()
            .map(|val| BasicMetadataValueEnum::from(*val))
            .collect();

        let fn_val = fn_id.get_from(env);
        let ret_val = self.builder.build_call(fn_val.ink(), &param_vals, "tmp")?;
        let ret_ptr = ret_val
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        let ret_type_id = fn_val.ret_type;

        Ok((ret_ptr, ret_type_id))
    }

    fn copy_pointer(
        &self,
        ptr: PointerValue<'ctx>,
        ptr_type_id: TypeId,
        env: &Environment<'ctx>,
    ) -> Result<PointerValue<'ctx>, GenError> {
        let ptr_type = env.get_type(ptr_type_id);
        let idx = ptr_type.ink().count_fields() - 1;

        let ref_count_ptr =
            self.builder
                .build_struct_gep(ptr_type.ink(), ptr, idx, "refcountptr")?;

        let ref_count = self
            .builder
            .build_load(self.ctx.i64_type(), ref_count_ptr, "refcount")?
            .into_int_value();
        let ref_count = self.builder.build_int_add(
            ref_count,
            self.ctx.i64_type().const_int(1, false),
            "refcount",
        )?;

        self.builder.build_store(ref_count_ptr, ref_count)?;

        Ok(ptr)
    }

    fn destroy_pointer(
        &self,
        ptr: PointerValue<'ctx>,
        ptr_type_id: TypeId,
        env: &Environment<'ctx>,
    ) -> Result<(), GenError> {
        let ptr_type = env.get_type(ptr_type_id);
        let idx = ptr_type.ink().count_fields() - 1;

        let ref_count_ptr =
            self.builder
                .build_struct_gep(ptr_type.ink(), ptr, idx, "refcountptr")?;

        let ref_count = self
            .builder
            .build_load(self.ctx.i64_type(), ref_count_ptr, "refcount")?
            .into_int_value();

        let new_ref_count = self.builder.build_int_sub(
            ref_count,
            self.ctx.i64_type().const_int(1, false),
            "new_refcount",
        )?;

        self.builder.build_store(ref_count_ptr, new_ref_count)?;

        let ref_count_zero = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            new_ref_count,
            self.ctx.i64_type().const_int(0, false),
            "is_zero",
        )?;

        let unalloc_block = self
            .ctx
            .insert_basic_block_after(self.builder.get_insert_block().unwrap(), "unalloc");
        let merge_block = self
            .ctx
            .insert_basic_block_after(self.builder.get_insert_block().unwrap(), "continue");

        self.builder
            .build_conditional_branch(ref_count_zero, unalloc_block, merge_block)?;

        self.builder.position_at_end(unalloc_block);
        self.builder.build_free(ptr)?;
        self.builder.build_unconditional_branch(merge_block)?;

        self.builder.position_at_end(merge_block);

        Ok(())
    }
}
