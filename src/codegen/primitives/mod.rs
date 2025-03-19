use inkwell::{
    types::StructType,
    values::{BasicValueEnum, FunctionValue},
};

use super::{env::Environment, err::GenError, CodeGen};

pub mod bool;
pub mod float;
pub mod int;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_primitive_types(
        &mut self,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.setup_int_primitive(env)?;
        self.setup_float_primitive(env)?;
        self.setup_bool_primitive(env)?;

        Ok(())
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

        let param_val_ptr =
            self.builder
                .build_struct_gep(prim_struct, param_ptr, 0, "param_ptr")?;

        Ok(self.builder.build_load(
            prim_struct.get_field_type_at_index(0).unwrap(),
            param_val_ptr,
            "param",
        )?)
    }

    fn build_primitive_binary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        left_type: StructType<'ctx>,
        right_type: StructType<'ctx>,
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

        self.builder.build_aggregate_return(&[result_val])?;
        Ok(())
    }

    fn build_primitive_unary_fn(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        expr_type: StructType<'ctx>,
        build_op_fn: impl FnOnce(
            &mut Self,
            BasicValueEnum<'ctx>,
        ) -> Result<BasicValueEnum<'ctx>, GenError>,
    ) -> Result<(), GenError> {
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let expr = self.get_primitive_from_param(0, fn_val, expr_type)?;

        let result_val = build_op_fn(self, expr)?;

        self.builder.build_aggregate_return(&[result_val])?;
        Ok(())
    }
}
