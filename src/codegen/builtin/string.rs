use inkwell::{
    builder::Builder,
    types::{IntType, PointerType, StructType},
    values::{IntValue, PointerValue},
    AddressSpace,
};

use crate::{
    codegen::{
        env::{
            id::{TypeId, BOOL_ID, STR_ID},
            type_def::TypeDef,
            Environment,
        },
        err::GenError,
        CodeGen,
    },
    parser::BinaryOp,
};

pub const STR_NAME: &str = "str";

impl<'ctx> CodeGen<'ctx> {
    pub fn declare_str_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let str_struct = self.create_struct_type(
            STR_NAME,
            vec![self.str_ptr_type().into(), self.len_type().into()],
        );
        let str_type = TypeDef::new_prim(STR_NAME, str_struct);

        env.reserve_type_id(STR_ID, true)?;
        env.register_type(STR_NAME, STR_ID, str_type)?;

        Ok(())
    }

    pub fn setup_str_primitive(&mut self, env: &mut Environment<'ctx>) -> Result<(), GenError> {
        let str_struct_type = STR_ID.get_from(env).ink();

        self.build_free_ptr_fn(STR_ID, str_unalloc, env)?;
        self.build_copy_ptr_fn(STR_ID, env)?;

        // Binary Functions
        self.setup_str_eq_str(str_struct_type, env)?;
        self.setup_str_add_str(str_struct_type, env)?;

        Ok(())
    }

    fn setup_str_eq_str(
        &mut self,
        str_struct_type: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(STR_ID),
            BinaryOp::Equal.fn_name(),
            &[STR_ID, STR_ID],
            BOOL_ID,
            false,
        )?;

        let int_type = self.len_type();
        let bool_struct_type = env.get_type(BOOL_ID).ink();

        self.build_binary_fn(fn_val, |gen, left, right| {
            let (left_str_ptr, left_str_len) = gen.build_extract_string(left, str_struct_type)?;
            let (right_str_ptr, right_str_len) =
                gen.build_extract_string(right, str_struct_type)?;

            let entry_block = gen.builder.get_insert_block().unwrap();
            let check_end_branch = gen.ctx.append_basic_block(fn_val, "check_end");
            let compare_branch = gen.ctx.append_basic_block(fn_val, "compare_str");
            let merge_branch = gen.ctx.append_basic_block(fn_val, "merge");

            gen.builder.position_at_end(merge_branch);
            let result_val = gen.builder.build_phi(gen.ctx.bool_type(), "merge_val")?;

            // Ensure string length is equal, otherwise false
            gen.builder.position_at_end(entry_block);
            let eq_len = gen.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                left_str_len,
                right_str_len,
                "str_len_eq",
            )?;
            gen.builder
                .build_conditional_branch(eq_len, check_end_branch, merge_branch)?;
            result_val.add_incoming(&[(&gen.ctx.bool_type().const_int(0, false), entry_block)]); // Not equal if merge

            // Check to see if were at end, if we are true
            gen.builder.position_at_end(check_end_branch);

            let index_phi = gen.builder.build_phi(int_type, "index")?;
            index_phi.add_incoming(&[(&int_type.const_int(0, false), entry_block)]);

            let index = index_phi.as_basic_value().into_int_value();

            let at_end = gen.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                index,
                left_str_len,
                "at_end",
            )?;

            gen.builder
                .build_conditional_branch(at_end, merge_branch, compare_branch)?;
            result_val
                .add_incoming(&[(&gen.ctx.bool_type().const_int(1, false), check_end_branch)]); // Equal if merge

            // Check if next characters are equal, otherwise false
            gen.builder.position_at_end(compare_branch);

            let (left_char, right_char) = unsafe {
                (
                    gen.build_extract_char(left_str_ptr, index)?,
                    gen.build_extract_char(right_str_ptr, index)?,
                )
            };

            let char_eq = gen.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                left_char,
                right_char,
                "char_eq",
            )?;

            let next_index =
                gen.builder
                    .build_int_add(index, int_type.const_int(1, false), "next_index")?;
            index_phi.add_incoming(&[(&next_index, compare_branch)]);

            gen.builder
                .build_conditional_branch(char_eq, check_end_branch, merge_branch)?;
            result_val.add_incoming(&[(&gen.ctx.bool_type().const_int(0, false), compare_branch)]); // Not equal if merge

            gen.builder.position_at_end(merge_branch);
            gen.build_struct(bool_struct_type, vec![result_val.as_basic_value()])
        })
    }

    fn setup_str_add_str(
        &mut self,
        str_struct_type: StructType<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(
            Some(STR_ID),
            BinaryOp::Add.fn_name(),
            &[STR_ID, STR_ID],
            STR_ID,
            false,
        )?;

        let char_type = self.char_type();

        self.build_binary_fn(fn_val, |gen, left, right| {
            let (left_str_ptr, left_str_len) = gen.build_extract_string(left, str_struct_type)?;
            let (right_str_ptr, right_str_len) =
                gen.build_extract_string(right, str_struct_type)?;

            let str_data_size =
                gen.builder
                    .build_int_add(left_str_len, right_str_len, "new_str_data_size")?;

            let str_data_ptr =
                gen.builder
                    .build_array_malloc(char_type, str_data_size, "new_str_data_ptr")?;

            // Copy left into string data
            // TODO: figure out if there's a better way to determine alignment
            gen.builder.build_memcpy(
                str_data_ptr,
                1, // dest_align_bytes
                left_str_ptr,
                1, // src_align_bytes
                left_str_len,
            )?;

            // Copy right into string data
            let str_data_ptr_to_right = unsafe {
                gen.builder.build_gep(
                    char_type,
                    str_data_ptr,
                    &[left_str_len],
                    "str_data_ptr_to_right",
                )?
            };
            gen.builder.build_memcpy(
                str_data_ptr_to_right,
                1, // dest_align_bytes
                right_str_ptr,
                1, // src_align_bytes
                right_str_len,
            )?;

            gen.build_struct(
                str_struct_type,
                vec![str_data_ptr.into(), str_data_size.into()],
            )
        })
    }

    fn build_extract_string(
        &self,
        struct_ptr: PointerValue<'ctx>,
        str_type: StructType<'ctx>,
    ) -> Result<(PointerValue<'ctx>, IntValue<'ctx>), GenError> {
        let str_ptr_ptr = self
            .builder
            .build_struct_gep(str_type, struct_ptr, 0, "param_ptr")?;

        let str_ptr = self
            .builder
            .build_load(self.str_ptr_type(), str_ptr_ptr, "str_data")?
            .into_pointer_value();

        let len_ptr = self
            .builder
            .build_struct_gep(str_type, struct_ptr, 1, "param_len")?;

        let str_len = self
            .builder
            .build_load(self.len_type(), len_ptr, "str_len")?
            .into_int_value();

        Ok((str_ptr, str_len))
    }

    unsafe fn build_extract_char(
        &self,
        str_ptr: PointerValue<'ctx>,
        idx: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, GenError> {
        let char_type = self.char_type();
        let char_ptr = self
            .builder
            .build_gep(char_type, str_ptr, &[idx], "char_ptr")?;

        Ok(self
            .builder
            .build_load(char_type, char_ptr, "char_val")?
            .into_int_value())
    }

    fn char_type(&self) -> IntType<'ctx> {
        self.ctx.i8_type()
    }

    fn len_type(&self) -> IntType<'ctx> {
        self.ctx.i64_type()
    }

    fn str_ptr_type(&self) -> PointerType<'ctx> {
        self.ctx.ptr_type(AddressSpace::default())
    }
}

fn str_unalloc(
    ptr: PointerValue<'_>,
    type_id: TypeId,
    builder: &mut Builder<'_>,
    env: &mut Environment<'_>,
) -> Result<(), GenError> {
    let ink_type = type_id.get_from(env).ink();
    let ptr_ptr_str = builder.build_struct_gep(ink_type, ptr, 0, "ptr_ptr_str_data")?;
    let ptr_str = builder
        .build_load(
            ink_type.get_field_type_at_index(0).unwrap(),
            ptr_ptr_str,
            "ptr_str_data",
        )?
        .into_pointer_value();
    builder.build_free(ptr_str)?;
    Ok(())
}
