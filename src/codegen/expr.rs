use std::ops::Deref;

use inkwell::{values::PointerValue, AddressSpace};

use crate::parser::{BinaryOp, Expression, Primary, UnaryOp};

use super::{
    env::{
        id::{TypeId, BOOL_ID, FLOAT_ID, INT_ID, STR_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn compile_expression(
        &mut self,
        expression: &Expression,
        env: &mut Environment<'ctx>,
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
        env: &mut Environment<'ctx>,
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
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (left_ptr, left_type) = self.compile_expression(left, env)?;
        let (right_ptr, right_type) = self.compile_expression(right, env)?;

        let op_func_name = op.fn_name();
        let op_func_id = env.find_func(op_func_name, Some(left_type), &[left_type, right_type])?;

        let ret = self.call_func(op_func_id, &[left_ptr, right_ptr], env)?;

        self.free_pointer(left_ptr, left_type, env)?;
        self.free_pointer(right_ptr, right_type, env)?;

        Ok(ret)
    }

    fn compile_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Box<Expression>,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_type) = self.compile_expression(expr, env)?;

        let op_func_id = env.find_func(op.fn_name(), Some(expr_type), &[expr_type])?;

        let ret = self.call_func(op_func_id, &[expr_ptr], env)?;

        self.free_pointer(expr_ptr, expr_type, env)?;

        Ok(ret)
    }

    pub(super) fn compile_primary(
        &mut self,
        primary: &Primary,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        match primary {
            Primary::Identifier(ident) => {
                let (var_ptr, type_id) = env.get_var(ident)?;
                let expr_ptr = self
                    .builder
                    .build_load(
                        self.ctx.ptr_type(AddressSpace::default()),
                        var_ptr,
                        &(ident.to_owned() + "_val"),
                    )?
                    .into_pointer_value();

                let ptr = self.copy_pointer(expr_ptr, type_id, env)?;
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
            Primary::String(val) => {
                let string_type = env.get_type(STR_ID);

                // let str_len = val.len() + 1;
                let char_type = self.ctx.i8_type();
                let array_type = char_type.array_type(val.len() as u32);
                let ptr_str_data = self.builder.build_malloc(array_type, "ptr_str_data")?;
                let str_data = char_type.const_array(
                    &val.bytes()
                        .map(|byte| char_type.const_int(byte as u64, false))
                        .collect::<Vec<_>>(),
                );
                self.builder.build_store(ptr_str_data, str_data)?;

                let str_size = self.ctx.i64_type().const_int(val.len() as u64, false);
                let ptr = self.build_struct(
                    string_type.ink(),
                    vec![ptr_str_data.into(), str_size.into()],
                )?;

                Ok((ptr, STR_ID))
            }
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
}
