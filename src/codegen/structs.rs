use inkwell::{
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicValueEnum, PointerValue},
};

use crate::codegen::ink_extension::{InkTypeExt, InkValueExt};

use super::{env::Environment, err::GenError, CodeGen};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn compile_struct_definition(
        &mut self,
        identifier: &str,
        fields: &[(String, String)],
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let field_types = fields
            .iter()
            .map(|(_, type_ident)| env.find_type(type_ident))
            .collect::<Result<Vec<_>, GenError>>()?;

        let field_inks = field_types
            .iter()
            .map(|id| env.get_type(*id).ink().as_basic_type_enum())
            .collect();

        let struct_type = self.create_struct_type(identifier, field_inks);

        todo!()
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

        for i in 1..struct_type.count_fields() {
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
}
