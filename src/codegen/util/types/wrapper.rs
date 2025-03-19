#[macro_export]
macro_rules! wrapper_type {
    ($name:ident: $type:ident) => {
        use crate::codegen::util::types::{RetType, TypeEnum};
        use inkwell::types;
        use inkwell::types::{BasicType, BasicTypeEnum};

        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $name<'ctx> {
            value: types::$type<'ctx>,
        }

        // impl<'ctx> Deref for $name<'ctx> {
        //     type Target = types::$type<'ctx>;

        //     fn deref(&self) -> &Self::Target {
        //         &self.value
        //     }
        // }

        impl<'ctx> $name<'ctx> {
            pub(crate) fn new(value: types::$type<'ctx>) -> Self {
                Self { value }
            }
        }

        impl<'ctx> RetType<'ctx> for $name<'ctx> {
            fn as_type_enum(&self) -> TypeEnum<'ctx> {
                TypeEnum::$name(self.clone())
            }

            fn try_as_inkwell_basic(&self) -> Result<BasicTypeEnum<'ctx>, String> {
                Ok(self.value.as_basic_type_enum())
            }
        }
    };
}
