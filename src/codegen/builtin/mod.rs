use inkwell::{
    types::{PointerType, StructType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use llvm_resources::LLVMResources;

use super::{
    env::{
        id::{TypeId, STR_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

pub mod bool;
pub mod float;
pub mod functions;
pub mod int;
pub mod llvm_resources;
pub mod none;
pub mod string;

pub const TO_STR_FN: &str = "__str__";
pub const TO_BOOL_FN: &str = "__bool__";
pub const TO_INT_FN: &str = "__int__";
pub const TO_FLOAT_FN: &str = "__float__";

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_builtins(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let res = self.setup_llvm_resources(env)?;

        self.declare_int_primitive(env)?;
        self.declare_float_primitive(env)?;
        self.declare_bool_primitive(env)?;
        self.declare_none_primitive(env)?;
        self.declare_str_primitive(env)?;

        self.setup_int_primitive(&res, env)?;
        self.setup_float_primitive(&res, env)?;
        self.setup_bool_primitive(&res, env)?;
        self.setup_none_primitive(env)?;
        self.setup_str_primitive(&res, env)?;

        self.setup_functions(&res, env)?;

        Ok(())
    }

    pub fn extract_primitive(
        &mut self,
        struct_ptr: PointerValue<'ctx>,
        struct_type: StructType<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, GenError> {
        let val_ptr = self
            .builder
            .build_struct_gep(struct_type, struct_ptr, 0, "param_ptr")?;

        Ok(self.builder.build_load(
            struct_type.get_field_type_at_index(0).unwrap(),
            val_ptr,
            "primitive",
        )?)
    }

    fn create_binary_fn(
        &mut self,
        ident: &str,
        left_tid: TypeId,
        right_tid: TypeId,
        ret_tid: TypeId,
        should_free: bool,
        build_op_fn: impl FnOnce(
            &mut Self,
            PointerValue<'ctx>,
            PointerValue<'ctx>,
            &mut Environment<'ctx>,
        ) -> Result<PointerValue<'ctx>, GenError>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(left_tid),
            ident,
            &[left_tid, right_tid],
            ret_tid,
            false,
        )?;

        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let left = fn_val.get_nth_param(0).unwrap().into_pointer_value();
        let right = fn_val.get_nth_param(1).unwrap().into_pointer_value();

        let result_ptr = build_op_fn(self, left, right, env)?;

        if should_free {
            self.free_pointer(left, left_tid, env)?;
            self.free_pointer(right, right_tid, env)?;
        }

        self.builder.build_return(Some(&result_ptr))?;
        Ok(())
    }

    fn create_primitive_binary_fn(
        &mut self,
        ident: &str,
        left_tid: TypeId,
        right_tid: TypeId,
        ret_tid: TypeId,
        build_op_fn: impl FnOnce(
            &mut Self,
            BasicValueEnum<'ctx>,
            BasicValueEnum<'ctx>,
        ) -> Result<BasicValueEnum<'ctx>, GenError>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let left_type = env.get_type(left_tid).ink();
        let right_type = env.get_type(right_tid).ink();
        let ret_type = env.get_type(ret_tid).ink();

        self.create_binary_fn(
            ident,
            left_tid,
            right_tid,
            ret_tid,
            true,
            |gen, left, right, _env| {
                let left_prim = gen.extract_primitive(left, left_type)?;
                let right_prim = gen.extract_primitive(right, right_type)?;

                let result_val = build_op_fn(gen, left_prim, right_prim)?;

                gen.build_struct(ret_type, vec![result_val])
            },
            env,
        )
    }

    fn create_unary_fn(
        &mut self,
        ident: &str,
        param_tid: TypeId,
        ret_tid: TypeId,
        should_free: bool,
        build_op_fn: impl FnOnce(
            &mut Self,
            FunctionValue<'ctx>,
            PointerValue<'ctx>,
            &mut Environment<'ctx>,
        ) -> Result<PointerValue<'ctx>, GenError>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(param_tid), ident, &[param_tid], ret_tid, false)?;

        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let param = fn_val.get_nth_param(0).unwrap().into_pointer_value();

        let result_ptr = build_op_fn(self, fn_val, param, env)?;

        if should_free {
            self.free_pointer(param, param_tid, env)?;
        }

        self.builder.build_return(Some(&result_ptr))?;
        Ok(())
    }

    fn create_primitive_unary_fn(
        &mut self,
        ident: &str,
        param_tid: TypeId,
        ret_tid: TypeId,
        build_op_fn: impl FnOnce(
            &mut Self,
            BasicValueEnum<'ctx>,
        ) -> Result<BasicValueEnum<'ctx>, GenError>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let expr_type = env.get_type(param_tid).ink();
        let ret_type = env.get_type(ret_tid).ink();

        self.create_unary_fn(
            ident,
            param_tid,
            ret_tid,
            true,
            |gen, _fn_val, param, _env| {
                let param_prim = gen.extract_primitive(param, expr_type)?;
                let result_val = build_op_fn(gen, param_prim)?;
                gen.build_struct(ret_type, vec![result_val])
            },
            env,
        )
    }

    fn create_primitive_to_str_fn(
        &mut self,
        tid: TypeId,
        format_spec_str: &str,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let type_def = env.get_type(tid);
        let expr_type = type_def.ink();
        let type_ident = type_def.ident().to_owned();

        let format_spec = self
            .builder
            .build_global_string_ptr(format_spec_str, &(type_ident + "_format_specifier"))?
            .as_pointer_value();

        self.create_unary_fn(
            TO_STR_FN,
            tid,
            STR_ID,
            true,
            |gen, _fn_val, param, env| {
                let prim = gen.extract_primitive(param, expr_type)?;

                let str_size = gen.build_get_string_size(format_spec, prim, res)?;
                let cstr_size = gen.builder.build_int_add(
                    str_size,
                    gen.len_type().const_int(1, false),
                    "str_size",
                )?;

                let str_data_ptr = gen.build_str_data_malloc(cstr_size, "new_str_data_ptr")?;

                // load data into str
                gen.builder.build_call(
                    res.snprintf,
                    &[
                        str_data_ptr.into(),
                        cstr_size.into(),
                        format_spec.into(),
                        prim.into(),
                    ],
                    "_",
                )?;

                gen.build_str_struct(str_data_ptr, str_size, env)
            },
            env,
        )
    }

    pub(super) fn ptr_type(&self) -> PointerType<'ctx> {
        self.ctx.ptr_type(AddressSpace::default())
    }
}

fn primitive_unalloc(
    _ptr: PointerValue<'_>,
    _type_id: TypeId,
    _builder: &mut CodeGen<'_>,
    _env: &mut Environment<'_>,
) -> Result<(), GenError> {
    Ok(())
}
