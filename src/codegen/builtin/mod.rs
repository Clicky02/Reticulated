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
        self.declare_int_primitive(env)?;
        self.declare_float_primitive(env)?;
        self.declare_bool_primitive(env)?;
        self.declare_none_primitive(env)?;
        self.declare_str_primitive(env)?;

        self.setup_int_primitive(env)?;
        self.setup_float_primitive(env)?;
        self.setup_bool_primitive(env)?;
        self.setup_none_primitive(env)?;
        self.setup_str_primitive(env)?;

        self.setup_functions(env)?;

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

    fn get_primitive_from_param(
        &mut self,
        param_idx: u32,
        fn_val: FunctionValue<'ctx>,
        prim_struct: StructType<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, GenError> {
        let param_ptr = fn_val
            .get_nth_param(param_idx)
            .unwrap()
            .into_pointer_value();

        self.extract_primitive(param_ptr, prim_struct)
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
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let expr = self.get_primitive_from_param(0, fn_val, expr_type)?;

        let result_val = build_op_fn(self, expr)?;

        let ptr = self.build_struct(ret_type, vec![result_val])?;
        self.builder.build_return(Some(&ptr))?;
        Ok(())
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
