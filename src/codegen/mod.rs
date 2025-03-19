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
pub mod to_meta;

use crate::parser::{BinaryOp, Expression, Primary, Statement};

const INT_NAME: &str = "int";
const FLOAT_NAME: &str = "float";
const BOOL_NAME: &str = "bool";

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

    fn setup_primitive_types(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let int_struct = self.ctx.opaque_struct_type(INT_NAME);
        int_struct.set_body(&[self.ctx.i64_type().into()], false);
        env.reserve_type_id(INT_ID, INT_NAME, int_struct)?;

        let int_add_int_fn =
            env.create_func(Some(INT_ID), "__add__", &[INT_ID, INT_ID], INT_ID, false)?;

        let int_add_int_entry = self.ctx.append_basic_block(int_add_int_fn, "entry");
        self.builder.position_at_end(int_add_int_entry);

        let left_ptr = int_add_int_fn
            .get_first_param()
            .unwrap()
            .into_pointer_value();
        let left = self
            .builder
            .build_struct_gep(int_struct, left_ptr, 0, "left_ptr")?;
        let left = self
            .builder
            .build_load(int_struct.get_field_type_at_index(0).unwrap(), left, "left")?
            .into_int_value();

        let right_ptr = int_add_int_fn
            .get_nth_param(1)
            .unwrap()
            .into_pointer_value();
        let right = self
            .builder
            .build_struct_gep(int_struct, right_ptr, 0, "right_ptr")?;
        let right = self
            .builder
            .build_load(
                int_struct.get_field_type_at_index(0).unwrap(),
                right,
                "right",
            )?
            .into_int_value();

        let result_val = self.builder.build_int_add(left, right, "result_val")?;
        self.builder.build_aggregate_return(&[result_val.into()])?;

        let float_struct = self.ctx.opaque_struct_type(FLOAT_NAME);
        float_struct.set_body(&[self.ctx.f64_type().into()], false);
        env.reserve_type_id(FLOAT_ID, FLOAT_NAME, float_struct)?;

        let bool_struct = self.ctx.opaque_struct_type(BOOL_NAME);
        bool_struct.set_body(&[self.ctx.bool_type().into()], false);
        env.reserve_type_id(BOOL_ID, BOOL_NAME, bool_struct)?;

        Ok(())
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
                condition: _,
                then_branch: _,
                else_if_branches: _,
                else_branch: _,
            } => todo!(),
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
            Expression::Unary(..) => todo!(),
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

        let op_func_name = op.to_op_func();
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
            Primary::Grouping(..) => todo!(),
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
