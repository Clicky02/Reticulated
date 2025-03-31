use c_functions::CFunctions;
use inkwell::{
    builder::Builder,
    types::StructType,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};

use super::{
    env::{id::TypeId, Environment},
    err::GenError,
    CodeGen,
};

pub mod bool;
pub mod c_functions;
pub mod float;
pub mod functions;
pub mod int;
pub mod none;
pub mod string;

pub const TO_STR_FN: &str = "__str__";
pub const TO_BOOL_FN: &str = "__bool__";
pub const TO_INT_FN: &str = "__int__";
pub const TO_FLOAT_FN: &str = "__float__";

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_builtins(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let cfns = self.setup_llvm_c_functions(env);

        self.declare_int_primitive(env)?;
        self.declare_float_primitive(env)?;
        self.declare_bool_primitive(env)?;
        self.declare_none_primitive(env)?;
        self.declare_str_primitive(env)?;

        self.setup_int_primitive(&cfns, env)?;
        self.setup_float_primitive(&cfns, env)?;
        self.setup_bool_primitive(&cfns, env)?;
        self.setup_none_primitive(env)?;
        self.setup_str_primitive(env)?;

        self.setup_functions(&cfns, env)?;

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

    fn build_binary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        build_op_fn: impl FnOnce(
            &mut Self,
            PointerValue<'ctx>,
            PointerValue<'ctx>,
        ) -> Result<PointerValue<'ctx>, GenError>,
    ) -> Result<(), GenError> {
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let left = fn_val.get_nth_param(0).unwrap().into_pointer_value();
        let right = fn_val.get_nth_param(1).unwrap().into_pointer_value();

        let result_ptr = build_op_fn(self, left, right)?;

        self.builder.build_return(Some(&result_ptr))?;
        Ok(())
    }

    fn build_primitive_binary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        left_type: StructType<'ctx>,
        right_type: StructType<'ctx>,
        ret_type: StructType<'ctx>,
        build_op_fn: impl FnOnce(
            &mut Self,
            BasicValueEnum<'ctx>,
            BasicValueEnum<'ctx>,
        ) -> Result<BasicValueEnum<'ctx>, GenError>,
    ) -> Result<(), GenError> {
        self.build_binary_fn(fn_val, |gen, left, right| {
            let left = gen.extract_primitive(left, left_type)?;
            let right = gen.extract_primitive(right, right_type)?;
            let result_val = build_op_fn(gen, left, right)?;
            gen.build_struct(ret_type, vec![result_val])
        })
    }

    fn build_unary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        build_op_fn: impl FnOnce(&mut Self, PointerValue<'ctx>) -> Result<PointerValue<'ctx>, GenError>,
    ) -> Result<(), GenError> {
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let param = fn_val.get_nth_param(0).unwrap().into_pointer_value();

        let result_ptr = build_op_fn(self, param)?;

        self.builder.build_return(Some(&result_ptr))?;
        Ok(())
    }

    fn build_primitive_unary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        expr_type: StructType<'ctx>,
        ret_type: StructType<'ctx>,
        build_op_fn: impl FnOnce(
            &mut Self,
            BasicValueEnum<'ctx>,
        ) -> Result<BasicValueEnum<'ctx>, GenError>,
    ) -> Result<(), GenError> {
        self.build_unary_fn(fn_val, |gen, param| {
            let param = gen.extract_primitive(param, expr_type)?;
            let result_val = build_op_fn(gen, param)?;
            gen.build_struct(ret_type, vec![result_val])
        })
    }

    fn build_primitive_to_str_fn(
        &mut self,
        type_name: &str,
        fn_val: FunctionValue<'ctx>,
        expr_type: StructType<'ctx>,
        format_spec_str: &str,
        cfns: &CFunctions<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let format_spec = self
            .builder
            .build_global_string_ptr(
                format_spec_str,
                &(type_name.to_owned() + "_format_specifier"),
            )?
            .as_pointer_value();

        self.build_unary_fn(fn_val, |gen, param| {
            let prim = gen.extract_primitive(param, expr_type)?;

            let str_size = gen.build_get_string_size(format_spec, prim, cfns)?;
            let cstr_size = gen.builder.build_int_add(
                str_size,
                gen.len_type().const_int(1, false),
                "str_size",
            )?;

            let str_data_ptr = gen.build_str_data_malloc(cstr_size, "new_str_data_ptr")?;

            // load data into str
            gen.builder.build_call(
                cfns.snprintf,
                &[
                    str_data_ptr.into(),
                    cstr_size.into(),
                    format_spec.into(),
                    prim.into(),
                ],
                "_",
            )?;

            gen.build_str_struct(str_data_ptr, str_size, env)
        })
    }
}

fn primitive_unalloc(
    _ptr: PointerValue<'_>,
    _type_id: TypeId,
    _builder: &mut Builder<'_>,
    _env: &mut Environment<'_>,
) -> Result<(), GenError> {
    Ok(())
}
