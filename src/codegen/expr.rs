use std::ops::Deref;

use inkwell::{values::PointerValue, AddressSpace};

use crate::parser::{BinaryFnOp, BinaryOp, Expression, Primary, UnaryFnOp, UnaryOp};

use super::{
    builtin::{TO_BOOL_FN, TO_FLOAT_FN, TO_INT_FN, TO_STR_FN},
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
        match expression {
            Expression::Binary(left, op, right) => self.compile_binary(left, op, right, env),
            Expression::BinaryFn(left, op, right) => self.compile_binary_fn(left, op, right, env),
            Expression::Unary(op, expr) => self.compile_unary(op, expr, env),
            Expression::UnaryFn(op, expr) => self.compile_unary_fn(op, expr, env),
            Expression::Invoke(expr, params) => self.compile_invoke(expr, params, env),
            Expression::Access(expr, id) => self.compile_access(expr, id, env),
            Expression::Primary(primary) => self.compile_primary(primary, env),
        }
    }

    pub(super) fn compile_access(
        &mut self,
        expr: &Box<Expression>,
        ident: &String,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_tid) = self.compile_expression(expr, env)?;
        let (field_ptr, field_tid) = self.build_gep_field(expr_ptr, expr_tid, ident, env)?;

        let field_val = self
            .builder
            .build_load(self.ctx.ptr_type(AddressSpace::default()), field_ptr, "_")?
            .into_pointer_value();

        Ok((field_val, field_tid))
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
        let (param_vals, param_tids): (Vec<_>, Vec<_>) = params.into_iter().unzip();

        let fn_id = match ident.as_str() {
            "str" => env.find_func(TO_STR_FN, param_tids.get(0).copied(), &param_tids)?,
            "int" => env.find_func(TO_INT_FN, param_tids.get(0).copied(), &param_tids)?,
            "float" => env.find_func(TO_FLOAT_FN, param_tids.get(0).copied(), &param_tids)?,
            "bool" => env.find_func(TO_BOOL_FN, param_tids.get(0).copied(), &param_tids)?,
            _ => env.find_func(ident, None, &param_tids)?,
        };

        self.call_func(fn_id, &param_vals, env)
    }

    // Currently, this only has short circuiting ops, everything else is implemented as a fn
    fn compile_binary(
        &mut self,
        left: &Box<Expression>,
        op: &BinaryOp,
        right: &Box<Expression>,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (left_ptr, left_tid) = self.compile_expression(left, env)?;

        if left_tid != BOOL_ID {
            return Err(GenError::InvalidType);
        }

        let left_type = left_tid.get_from(env).ink();
        let left_bool = self
            .extract_primitive(left_ptr, left_type)?
            .into_int_value();

        let cur_block = self.builder.get_insert_block().unwrap();
        let cur_fn = cur_block.get_parent().unwrap();
        let right_block = self
            .ctx
            .append_basic_block(cur_fn, &format!("right_{}_condition", op.to_string()));
        let continue_block = self.ctx.append_basic_block(cur_fn, "continue");

        match op {
            BinaryOp::And => {
                self.builder
                    .build_conditional_branch(left_bool, right_block, continue_block)?;
            }
            BinaryOp::Or => {
                self.builder
                    .build_conditional_branch(left_bool, continue_block, right_block)?;
            }
        };

        self.builder.position_at_end(right_block);
        self.free_pointer(left_ptr, left_tid, env)?;
        let (right_ptr, right_tid) = self.compile_expression(right, env)?;
        if right_tid != BOOL_ID {
            return Err(GenError::InvalidType);
        }
        self.builder.build_unconditional_branch(continue_block)?;

        self.builder.position_at_end(continue_block);
        let ret = self
            .builder
            .build_phi(self.ptr_type(), &format!("{}_result", op.to_string()))?;
        ret.add_incoming(&[(&left_ptr, cur_block), (&right_ptr, right_block)]);

        Ok((ret.as_basic_value().into_pointer_value(), BOOL_ID))
    }

    fn compile_binary_fn(
        &mut self,
        left: &Box<Expression>,
        op: &BinaryFnOp,
        right: &Box<Expression>,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (left_ptr, left_tid) = self.compile_expression(left, env)?;
        let (right_ptr, right_tid) = self.compile_expression(right, env)?;

        let op_func_name = op.fn_name();
        let op_func_id = env.find_func(op_func_name, Some(left_tid), &[left_tid, right_tid])?;

        let ret = self.call_func(op_func_id, &[left_ptr, right_ptr], env)?;

        self.free_pointer(left_ptr, left_tid, env)?;
        self.free_pointer(right_ptr, right_tid, env)?;

        Ok(ret)
    }

    fn compile_unary(
        &mut self,
        op: &UnaryOp,
        expr: &Box<Expression>,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_tid) = self.compile_expression(expr, env)?;

        if expr_tid != BOOL_ID {
            return Err(GenError::InvalidType);
        }

        let bool_type = BOOL_ID.get_from(env).ink();
        let expr_bool = self
            .extract_primitive(expr_ptr, bool_type)?
            .into_int_value();
        self.free_pointer(expr_ptr, expr_tid, env)?;

        match op {
            UnaryOp::Not => {
                let ret = self
                    .builder
                    .build_not(expr_bool, &(op.to_string().to_owned() + "_result"))?;
                let ret_struct_ptr = self.build_struct(bool_type, vec![ret.into()])?;
                Ok((ret_struct_ptr, BOOL_ID))
            }
        }
    }

    fn compile_unary_fn(
        &mut self,
        op: &UnaryFnOp,
        expr: &Box<Expression>,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let (expr_ptr, expr_tid) = self.compile_expression(expr, env)?;

        let op_func_id = env.find_func(op.fn_name(), Some(expr_tid), &[expr_tid])?;

        let ret = self.call_func(op_func_id, &[expr_ptr], env)?;

        self.free_pointer(expr_ptr, expr_tid, env)?;

        Ok(ret)
    }

    pub(super) fn compile_primary(
        &mut self,
        primary: &Primary,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        match primary {
            Primary::Identifier(ident) => {
                let (var_ptr, tid) = env.get_var(ident)?;
                let expr_ptr = self
                    .builder
                    .build_load(
                        self.ctx.ptr_type(AddressSpace::default()),
                        var_ptr,
                        &(ident.to_owned() + "_val"),
                    )?
                    .into_pointer_value();

                let ptr = self.copy_pointer(expr_ptr, tid, env)?;
                Ok((ptr, tid))
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
                let ptr = self.build_str_const(val, env)?;
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

    pub(super) fn build_gep_field(
        &mut self,
        struct_ptr: PointerValue<'ctx>,
        struct_tid: TypeId,
        field_id: &String,
        env: &mut Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let type_def = env.get_type(struct_tid);
        let struct_type = type_def.ink();

        let field = type_def.find_field(&field_id)?;
        let field_ptr = self.builder.build_struct_gep(
            struct_type,
            struct_ptr,
            field.index(),
            &(field_id.to_owned() + "_field"),
        )?;

        Ok((field_ptr, field.tid()))
    }
}
