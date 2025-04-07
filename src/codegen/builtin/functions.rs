use crate::codegen::{
    env::{
        id::{NONE_ID, STR_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

use super::llvm_resources::LLVMResources;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn setup_functions(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        self.setup_print(res, env)?;
        self.setup_input(res, env)?;

        Ok(())
    }

    fn setup_print(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let str_type = env.get_type(STR_ID).ink();

        let (print_fn, ..) = env.create_func(None, "print", &[STR_ID], NONE_ID, false)?;

        let entry = self.ctx.append_basic_block(print_fn, "entry");
        self.builder.position_at_end(entry);

        let str_struct_ptr = print_fn.get_nth_param(0).unwrap().into_pointer_value();
        let (str_ptr, str_len) = self.build_extract_string(str_struct_ptr, str_type)?;

        let format_spec = self
            .builder
            .build_global_string_ptr("%.*s\n", "print_string_format")?
            .as_pointer_value();

        self.builder.build_call(
            res.printf,
            &[format_spec.into(), str_len.into(), str_ptr.into()],
            "_",
        )?;

        let none_value = self.build_none(env)?;
        self.builder.build_return(Some(&none_value))?;

        Ok(())
    }

    fn setup_input(
        &mut self,
        res: &LLVMResources<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let str_type = env.get_type(STR_ID).ink();
        let (input_fn, ..) = env.create_func(None, "input", &[STR_ID], STR_ID, false)?;

        let int_type = self.ctx.i32_type();
        let ptr_type = self.ptr_type();
        let char_type = self.char_type();

        let scan_format_spec = self
            .builder
            .build_global_string_ptr("%127[^\n]%n", "input_format")?
            .as_pointer_value();

        let entry = self.ctx.append_basic_block(input_fn, "entry");
        let loop_block = self.ctx.append_basic_block(input_fn, "loop");
        let return_block = self.ctx.append_basic_block(input_fn, "return");

        self.builder.position_at_end(entry);

        // Print Input
        let str_struct_ptr = input_fn.get_nth_param(0).unwrap().into_pointer_value();
        let (str_ptr, str_len) = self.build_extract_string(str_struct_ptr, str_type)?;
        self.builder.build_call(
            res.printf,
            &[res.str_format_spec.into(), str_len.into(), str_ptr.into()],
            "_",
        )?;

        // Create chunk buffer
        let chunk_buffer_size = self.len_type().const_int(128, false);
        let chunk_buffer =
            self.builder
                .build_array_alloca(char_type, chunk_buffer_size, "chunk_buffer")?;

        self.builder.build_unconditional_branch(loop_block)?;
        self.builder.position_at_end(loop_block);

        let total_size_phi = self.builder.build_phi(int_type, "total_size")?;
        let total_size = total_size_phi.as_basic_value().into_int_value();
        total_size_phi.add_incoming(&[(&int_type.const_zero(), entry)]);

        let buffer_phi = self.builder.build_phi(ptr_type, "output")?;
        let buffer = buffer_phi.as_basic_value().into_pointer_value();
        buffer_phi.add_incoming(&[(&ptr_type.const_null(), entry)]);

        let chars_read_ptr = self.builder.build_alloca(int_type, "chars_read_ptr")?;
        self.builder
            .build_store(chars_read_ptr, int_type.const_zero())?;

        // fetch from stdin
        self.builder
            .build_call(
                res.scanf,
                &[
                    scan_format_spec.into(),
                    chunk_buffer.into(),
                    chars_read_ptr.into(),
                ],
                "match_cnt",
            )?
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();

        let chars_read = self
            .builder
            .build_load(int_type, chars_read_ptr, "chars_read")?
            .into_int_value();

        let new_size = self
            .builder
            .build_int_add(total_size, chars_read, "new_size")?;
        total_size_phi.add_incoming(&[(&new_size, loop_block)]);

        let new_buffer = self
            .builder
            .build_call(res.realloc, &[buffer.into(), new_size.into()], "new_output")?
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        buffer_phi.add_incoming(&[(&new_buffer, loop_block)]);

        let buffer_cpy_ptr = unsafe {
            self.builder
                .build_gep(char_type, new_buffer, &[total_size], "output_cpy_ptr")?
        };
        self.builder
            .build_memcpy(buffer_cpy_ptr, 1, chunk_buffer, 1, chars_read)?;

        let finished_matching = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            chars_read,
            int_type.const_int(127, false),
            "finished_matching",
        )?;

        self.builder
            .build_conditional_branch(finished_matching, return_block, loop_block)?;

        self.builder.position_at_end(return_block);

        let final_size = self
            .builder
            .build_int_cast(new_size, self.prim_int_type(), "str_size")?;
        let new_str_ptr = self.build_str_struct(new_buffer, final_size, env)?;
        self.builder.build_return(Some(&new_str_ptr))?;

        Ok(())
    }

}
