use inkwell::values::{BasicMetadataValueEnum, PointerValue};

use super::{
    builtin::llvm_resources::LLVMResources,
    env::{
        func::Scope,
        id::{FunctionId, TypeId, NONE_ID},
        Environment,
    },
    err::GenError,
    CodeGen,
};

pub const FREE_PTR_IDENT: &str = "$freeptr";
pub const COPY_PTR_IDENT: &str = "$copyptr";

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn call_func(
        &mut self,
        fn_id: FunctionId,
        args: &[PointerValue<'ctx>],
        env: &Environment<'ctx>,
    ) -> Result<(PointerValue<'ctx>, TypeId), GenError> {
        let param_vals: Vec<_> = args
            .into_iter()
            .map(|val| BasicMetadataValueEnum::from(*val))
            .collect();

        let fn_val = fn_id.get_from(env);
        let ret_val = self.builder.build_call(fn_val.ink(), &param_vals, "_")?;
        let ret_ptr = ret_val
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        Ok((ret_ptr, fn_val.ret_type))
    }

    pub(super) fn copy_pointer(
        &mut self,
        ptr: PointerValue<'ctx>,
        ptr_tid: TypeId,
        env: &mut Environment<'ctx>,
    ) -> Result<PointerValue<'ctx>, GenError> {
        let fn_id = env.find_func(COPY_PTR_IDENT, Some(ptr_tid), &[ptr_tid])?;
        let (ret, ..) = self.call_func(fn_id, &[ptr], env)?;
        Ok(ret)
    }

    pub(super) fn build_copy_ptr_fn(
        &mut self,
        tid: TypeId,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(tid), COPY_PTR_IDENT, &[tid], tid, false)?;
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let ptr = fn_val.get_nth_param(0).unwrap().into_pointer_value();

        let type_def = tid.get_from(env);
        let idx = type_def.ink().count_fields() - 1;
        let ref_count_ptr =
            self.builder
                .build_struct_gep(type_def.ink(), ptr, idx, "refcountptr")?;

        let ref_count = self
            .builder
            .build_load(self.ctx.i64_type(), ref_count_ptr, "refcount")?
            .into_int_value();
        let ref_count = self.builder.build_int_add(
            ref_count,
            self.ctx.i64_type().const_int(1, false),
            "refcount",
        )?;

        self.builder.build_store(ref_count_ptr, ref_count)?;

        self.builder.build_return(Some(&ptr))?;

        Ok(())
    }

    pub(super) fn build_noop_copy_ptr_fn(
        &mut self,
        tid: TypeId,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(tid), COPY_PTR_IDENT, &[tid], tid, false)?;
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let ptr = fn_val.get_nth_param(0).unwrap().into_pointer_value();

        self.builder.build_return(Some(&ptr))?;

        Ok(())
    }

    pub(super) fn free_pointer(
        &mut self,
        ptr: PointerValue<'ctx>,
        tid: TypeId,
        env: &Environment<'ctx>,
    ) -> Result<(), GenError> {
        let fn_id = env.find_func(FREE_PTR_IDENT, Some(tid), &[tid])?;
        self.call_func(fn_id, &[ptr], env)?;
        Ok(())
    }

    pub(super) fn build_free_ptr_fn(
        &mut self,
        tid: TypeId,
        custom_unalloc: impl FnOnce(
            PointerValue<'ctx>,
            TypeId,
            &mut Self,
            &mut Environment<'ctx>,
        ) -> Result<(), GenError>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(tid), FREE_PTR_IDENT, &[tid], NONE_ID, false)?; // TODO: Optional return
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let ptr = fn_val.get_nth_param(0).unwrap().into_pointer_value();

        let type_def = tid.get_from(env);
        let idx = type_def.ink().count_fields() - 1;
        let ref_count_ptr =
            self.builder
                .build_struct_gep(type_def.ink(), ptr, idx, "refcountptr")?;

        let ref_count = self
            .builder
            .build_load(self.ctx.i64_type(), ref_count_ptr, "refcount")?
            .into_int_value();

        let new_ref_count = self.builder.build_int_sub(
            ref_count,
            self.ctx.i64_type().const_int(1, false),
            "new_refcount",
        )?;

        self.builder.build_store(ref_count_ptr, new_ref_count)?;

        let ref_count_zero = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            new_ref_count,
            self.ctx.i64_type().const_int(0, false),
            "is_zero",
        )?;

        let unalloc_block = self.ctx.insert_basic_block_after(entry, "unalloc");
        let merge_block = self.ctx.insert_basic_block_after(unalloc_block, "continue");

        self.builder
            .build_conditional_branch(ref_count_zero, unalloc_block, merge_block)?;

        self.builder.position_at_end(unalloc_block);

        custom_unalloc(ptr, tid, self, env)?;
        self.builder.build_free(ptr)?; // DO NOT USE MEMORY AFTER FREE

        self.builder.build_unconditional_branch(merge_block)?;

        self.builder.position_at_end(merge_block);

        let none = self.build_none(env)?;
        self.builder.build_return(Some(&none))?;

        Ok(())
    }

    pub(super) fn build_noop_free_ptr_fn(
        &mut self,
        tid: TypeId,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let (fn_val, ..) = env.create_func(Some(tid), FREE_PTR_IDENT, &[tid], NONE_ID, false)?; // TODO: Optional return
        let entry = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        let none = self.build_none(env)?;
        self.builder.build_return(Some(&none))?;

        Ok(())
    }

    pub(super) fn free_vars_in_scope(
        &mut self,
        scope: &Scope<'ctx>,
        env: &Environment<'ctx>,
    ) -> Result<(), GenError> {
        for (var, tid) in scope.variables().values() {
            let var_val = self
                .builder
                .build_load(self.ptr_type(), *var, "loaded_var")?
                .into_pointer_value();
            self.free_pointer(var_val, *tid, env)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn debug(
        &mut self,
        format: &str,
        val: BasicMetadataValueEnum<'ctx>,
        res: &LLVMResources<'ctx>,
    ) -> Result<(), GenError> {
        let debug_format_spec = self
            .builder
            .build_global_string_ptr(format, "debug")?
            .as_pointer_value();

        self.builder
            .build_call(res.printf, &[debug_format_spec.into(), val], "_")
            .unwrap();

        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn debug_print(
        &mut self,
        format: &str,
        res: &LLVMResources<'ctx>,
    ) -> Result<(), GenError> {
        let debug_format_spec = self
            .builder
            .build_global_string_ptr(format, "debug")?
            .as_pointer_value();

        self.builder
            .build_call(res.printf, &[debug_format_spec.into()], "_")
            .unwrap();

        Ok(())
    }
}
