use inkwell::{
    types::StructType,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};

use super::{env::Environment, err::GenError, CodeGen};

pub mod bool;
pub mod float;
pub mod int;
pub mod none;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_primitive_types(
        &mut self,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.declare_int_primitive(env)?;
        self.declare_float_primitive(env)?;
        self.declare_bool_primitive(env)?;
        self.declare_none_primitive(env)?;

        self.setup_int_primitive(env)?;
        self.setup_float_primitive(env)?;
        self.setup_bool_primitive(env)?;
        self.setup_none_primitive(env)?;

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
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let left = self.get_primitive_from_param(0, fn_val, left_type)?;
        let right = self.get_primitive_from_param(1, fn_val, right_type)?;

        let result_val = build_op_fn(self, left, right)?;

        let ptr = self.build_struct(ret_type, vec![result_val])?;
        self.builder.build_return(Some(&ptr))?;
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
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let expr = self.get_primitive_from_param(0, fn_val, expr_type)?;

        let result_val = build_op_fn(self, expr)?;

        let ptr = self.build_struct(ret_type, vec![result_val])?;
        self.builder.build_return(Some(&ptr))?;
        Ok(())
    }
}
