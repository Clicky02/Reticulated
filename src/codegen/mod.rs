use std::collections::HashMap;

use env::Environment;
use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use to_meta::{ToMetaTypeEnum, ToMetaValueEnum};

pub mod env;
pub mod to_meta;

use crate::parser::{BinaryOp, Expression, FuncParameter, Primary, Statement};

// TODO: Not pub
pub struct CodeGen<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        Self {
            ctx,
            module: ctx.create_module("main"),
            builder: ctx.create_builder(),
        }
    }

    pub fn gen_code_for(&mut self, program: Vec<Statement>) {
        let fn_type = self.ctx.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", fn_type, None);

        let entry = self.ctx.append_basic_block(main_fn, "entry");

        self.builder.position_at_end(entry);

        let mut env = Environment::new(main_fn);

        for statement in program {
            self.compile_statement(&statement, &mut env);
        }

        self.builder
            .build_return(Some(&self.ctx.i32_type().const_int(0, false)))
            .unwrap();
    }

    pub fn compile_statement(&mut self, statement: &Statement, env: &mut env::Environment<'ctx>) {
        match statement {
            Statement::Declaration {
                identifier,
                type_identifier,
                expression,
            } => {
                let var_type = self.ident_to_type(type_identifier, env);
                let var_ptr = self.builder.build_alloca(var_type, identifier).unwrap();

                let value = self.compile_expression(expression, env);
                self.builder.build_store(var_ptr, value).unwrap();
                env.insert_var(identifier.clone(), var_ptr, var_type);
            }
            Statement::Assignment {
                identifier,
                expression,
            } => {
                let (var_ptr, _var_type) = env.get_var(identifier).unwrap();
                let value = self.compile_expression(expression, env); // TODO: Type check?
                self.builder.build_store(var_ptr, value).unwrap();
            }
            Statement::FunctionDeclaration {
                identifier,
                parameters,
                return_identifier,
                body,
            } => todo!(),
            Statement::ExternFunctionDeclaration {
                identifier,
                parameters,
                return_identifier,
            } => {
                let (param_types, is_var_args) = self.params_to_types(parameters, env);
                let fn_type = self
                    .ident_to_type(&return_identifier, env)
                    .fn_type(&param_types, is_var_args);
                self.module
                    .add_function(&identifier, fn_type, Some(Linkage::External));
            }
            Statement::Expression(expression) => {
                self.compile_expression(expression, env);
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => todo!(),
            Statement::ReturnStatement { expression } => todo!(),
        }
    }

    pub fn compile_expression(
        &mut self,
        expression: &Expression,
        env: &mut env::Environment<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match expression {
            Expression::Binary(left, op, right) => self.compile_binary(left, op, right, env),
            Expression::Unary(op, expr) => todo!(),
            Expression::Invoke(callee, args) => self.compile_invoke(callee, args, env),
            Expression::Primary(primary) => self.compile_primary(primary, env),
        }
    }

    fn compile_invoke(
        &mut self,
        callee: &Box<Expression>,
        args: &Vec<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        let Expression::Primary(Primary::Identifier(ref callee)) = **callee else {
            todo!("Add support for first order functions.")
        };

        let fn_val = self.module.get_function(&callee).unwrap();
        let mut arg_values = Vec::new();

        for arg in args {
            arg_values.push(self.compile_expression(arg, env).to_meta_val());
        }

        self.builder
            .build_call(fn_val, &arg_values, "calltmp")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    fn compile_binary(
        &mut self,
        left: &Box<Expression>,
        op: &BinaryOp,
        right: &Box<Expression>,
        env: &mut env::Environment<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        let left = self.compile_expression(left, env);
        let right = self.compile_expression(right, env);

        match op {
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::NotEqual => todo!(),
            BinaryOp::Equal => todo!(),
            BinaryOp::Greater => todo!(),
            BinaryOp::GreaterEqual => todo!(),
            BinaryOp::Less => todo!(),
            BinaryOp::LessEqual => todo!(),
            BinaryOp::Add => self.compile_add(left, right),
            BinaryOp::Subtract => self.compile_sub(left, right),
            BinaryOp::Multiply => self.compile_mul(left, right),
            BinaryOp::Divide => self.compile_div(left, right),
            BinaryOp::Modulo => self.compile_mod(left, right),
        }
    }

    fn compile_primary(
        &mut self,
        primary: &Primary,
        env: &mut Environment<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match primary {
            Primary::Identifier(ident) => {
                let (var_ptr, var_type) = env.get_var(ident).unwrap();
                self.builder.build_load(var_type, var_ptr, ident).unwrap()
            }
            Primary::Integer(val) => self
                .ctx
                .i64_type()
                .const_int(*val as u64, true) // TODO: figure out negatives
                .as_basic_value_enum(),
            Primary::Float(_) => todo!(),
            Primary::String(val) => {
                let str_val = self.ctx.const_string(val.as_bytes(), false);
                self.builder
                    .build_global_string_ptr(&val, "temp_str")
                    .unwrap()
                    .as_pointer_value()
                    .as_basic_value_enum()
            }
            Primary::Bool(val) => self
                .ctx
                .bool_type()
                .const_int((*val) as u64, false)
                .as_basic_value_enum(),
            Primary::None => todo!(),
            Primary::Grouping(expression) => todo!(),
        }
    }

    fn ident_to_type(
        &mut self,
        type_ident: &str,
        env: &mut Environment<'ctx>,
    ) -> BasicTypeEnum<'ctx> {
        match type_ident {
            // TODO: Better type identifier handling
            "int" => self.ctx.i64_type().as_basic_type_enum(),
            "float" => self.ctx.f64_type().as_basic_type_enum(),
            "bool" => self.ctx.bool_type().as_basic_type_enum(),
            "str" => self
                .ctx
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
            _ => todo!("Add user defined types"),
        }
    }

    fn params_to_types(
        &mut self,
        params: &Vec<FuncParameter>,
        env: &mut Environment<'ctx>,
    ) -> (Vec<BasicMetadataTypeEnum<'ctx>>, bool) {
        let mut param_types = Vec::new();
        let mut var_args = false;

        for i in 0..param_types.len() {
            let param = &params[i];

            if var_args {
                panic!("Cannot have paramter following a var args parameter.")
            }

            if param.var_args {
                var_args = true;
            } else {
                param_types.push(
                    self.ident_to_type(&param.type_identifier, env)
                        .to_meta_type(),
                );
            }
        }

        (param_types, var_args)
    }

    fn compile_bool_and(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        use BasicValueEnum::*;
        match (left, right) {
            (StructValue(test), _) => {
                // test.
                todo!()
            }
            (IntValue(left), IntValue(right)) => self
                .builder
                .build_int_compare(inkwell::IntPredicate::SGE, left, right, "tmp")
                .unwrap()
                .as_basic_value_enum(),
            (l, r) => panic!(
                "Unsupported {} operation between {} and {}.",
                stringify!($name),
                l.get_type(),
                r.get_type()
            ),
        }
    }
}

macro_rules! compile_num_op_func {
    ($fn_name:ident, $name:ident, $int_op:ident, $float_op:ident) => {
        impl<'ctx> CodeGen<'ctx> {
            fn $fn_name(
                &mut self,
                left: BasicValueEnum<'ctx>,
                right: BasicValueEnum<'ctx>,
            ) -> BasicValueEnum<'ctx> {
                use BasicValueEnum::*;

                match (left, right) {
                    (IntValue(left), IntValue(right)) => self
                        .builder
                        .$int_op(left, right, concat!(stringify!($name), "tmp"))
                        .unwrap()
                        .as_basic_value_enum(),
                    (IntValue(i_val), FloatValue(f_val)) => {
                        let left = self
                            .builder
                            .build_signed_int_to_float(i_val, self.ctx.f64_type(), "casttmp")
                            .unwrap();

                        self.builder
                            .$float_op(left, f_val, concat!(stringify!($name), "tmp"))
                            .unwrap()
                            .as_basic_value_enum()
                    }
                    (FloatValue(left), IntValue(i_val)) => {
                        let right = self
                            .builder
                            .build_signed_int_to_float(i_val, self.ctx.f64_type(), "casttmp")
                            .unwrap();

                        self.builder
                            .$float_op(left, right, concat!(stringify!($name), "tmp"))
                            .unwrap()
                            .as_basic_value_enum()
                    }
                    (FloatValue(left), FloatValue(right)) => self
                        .builder
                        .$float_op(left, right, concat!(stringify!($name), "tmp"))
                        .unwrap()
                        .as_basic_value_enum(),
                    (StructValue(_struct_value), _) => {
                        todo!("User-defined type operation not yet supported")
                    }
                    (l, r) => panic!(
                        "Unsupported {} operation between {} and {}.",
                        stringify!($name),
                        l.get_type(),
                        r.get_type()
                    ),
                }
            }
        }
    };
}

compile_num_op_func!(compile_add, add, build_int_add, build_float_add);
compile_num_op_func!(compile_sub, subtact, build_int_sub, build_float_sub);
compile_num_op_func!(compile_mul, multiply, build_int_mul, build_float_mul);
compile_num_op_func!(compile_div, divide, build_int_signed_div, build_float_div);
compile_num_op_func!(compile_mod, modulo, build_int_signed_rem, build_float_rem);
