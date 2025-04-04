use inkwell::{
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicValueEnum, PointerValue},
};

use crate::codegen::ink_extension::{InkTypeExt, InkValueExt};

use super::{
    env::{id::TypeId, type_def::TypeDef, Environment},
    err::GenError,
    util::FREE_PTR_IDENT,
    CodeGen,
};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn preprocess_struct_definition(
        &mut self,
        ident: &str,
        fields: &[(String, String)],
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let field_types = fields
            .iter()
            .map(|(_, type_ident)| env.find_type(type_ident))
            .collect::<Result<Vec<_>, GenError>>()?;

        let field_ptr_types = field_types
            .iter()
            .map(|_| self.ptr_type().as_basic_type_enum())
            .collect();

        let struct_type = self.create_struct_type(ident, field_ptr_types);

        let type_id = env.gen_type_id();
        env.register_type(
            ident,
            type_id,
            TypeDef::new(ident, struct_type, field_types),
        )?;

        Ok(())
    }

    pub(super) fn compile_struct_definition(
        &mut self,
        ident: &str,
        _fields: &[(String, String)],
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let type_id = env.find_type(ident)?;
        let prev_block = self.builder.get_insert_block().unwrap();

        // Pointer Functions
        self.build_free_ptr_fn(type_id, Self::struct_unalloc, env)?;
        self.build_copy_ptr_fn(type_id, env)?;

        // Constructor
        self.build_struct_constructor(ident, type_id, env)?;

        self.builder.position_at_end(prev_block);
        Ok(())
    }

    pub(super) fn create_struct_type(
        &self,
        ident: &str,
        mut fields: Vec<BasicTypeEnum<'ctx>>,
    ) -> StructType<'ctx> {
        fields.push(self.ctx.i64_type().into());

        let struct_type = self.ctx.opaque_struct_type(ident);
        struct_type.set_body(&fields, false);
        struct_type
    }

    pub(super) fn build_struct(
        &mut self,
        struct_type: StructType<'ctx>,
        mut values: Vec<BasicValueEnum<'ctx>>,
    ) -> Result<PointerValue<'ctx>, GenError> {
        values.push(self.ctx.i64_type().const_int(1, false).into());
        assert_eq!(struct_type.count_fields() as usize, values.len());

        let const_values: Vec<BasicValueEnum<'ctx>> = values
            .iter()
            .map(|val| {
                if val.is_const() {
                    *val
                } else {
                    val.get_type().get_poison()
                }
            })
            .collect();

        let val_ptr = self.builder.build_malloc(struct_type, "literal")?; // TODO: Memory Mangement ???? MALLOC??? RC???
        self.builder
            .build_store(val_ptr, struct_type.const_named_struct(&const_values))?;

        for i in 0..(struct_type.count_fields() - 1) {
            let val = values[i as usize];
            if !val.is_const() {
                let struct_val_ptr =
                    self.builder
                        .build_struct_gep(struct_type, val_ptr, i, "struct_val_ptr")?;
                self.builder.build_store(struct_val_ptr, val)?;
            }
        }

        // for i in 0..struct_type.count_fields() {
        //     let struct_val_ptr =
        //         self.builder
        //             .build_struct_gep(struct_type, val_ptr, i, "struct_val_ptr")?;
        //     self.builder
        //         .build_store(struct_val_ptr, values[i as usize])?;
        // }

        Ok(val_ptr)
    }

    fn struct_unalloc(
        ptr: PointerValue<'ctx>,
        type_id: TypeId,
        gen: &mut CodeGen<'ctx>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let type_def = type_id.get_from(env);
        let ink_type = type_def.ink();
        let fields = type_def.fields();

        for i in 0..fields.len() {
            let field = fields[i];
            let free_id: super::env::id::FunctionId =
                env.find_func(FREE_PTR_IDENT, Some(field), &[field])?;
            let field_ptr_ptr =
                gen.builder
                    .build_struct_gep(ink_type, ptr, i as u32, "field_ptr_ptr")?;
            let field_ptr = gen
                .builder
                .build_load(gen.ptr_type(), field_ptr_ptr, "field_ptr")?
                .into_pointer_value();
            gen.call_func(free_id, &[field_ptr], env)?;
        }

        Ok(())
    }

    fn build_struct_constructor(
        &mut self,
        ident: &str,
        type_id: TypeId,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let type_def = type_id.get_from(env);
        let fields = type_def.fields().to_vec();
        let struct_type = type_def.ink();

        let (fn_val, ..) = env.create_func(None, ident, &fields, type_id, false)?;
        let entry_block = self.ctx.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_block);

        let mut values = vec![];
        for i in 0..fields.len() {
            values.push(fn_val.get_nth_param(i as u32).unwrap());
        }
        let struct_ptr = self.build_struct(struct_type, values)?;
        self.builder.build_return(Some(&struct_ptr))?;

        Ok(())
    }
}
