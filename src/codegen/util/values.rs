use std::ops::Deref;

use inkwell::values;

use super::types::TypeEnum;

macro_rules! wrapper_value_type {
    ($name:ident: $type:ident) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $name<'ctx> {
            pub(super) value: values::$type<'ctx>,
        }

        impl<'ctx> Deref for $name<'ctx> {
            type Target = values::$type<'ctx>;

            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }

        impl<'ctx> $name<'ctx> {
            pub(super) fn new(value: values::$type<'ctx>) -> Self {
                Self { value }
            }
        }

        impl<'ctx> From<values::$type<'ctx>> for $name<'ctx> {
            fn from(value: values::$type<'ctx>) -> Self {
                $name::new(value)
            }
        }
    };
}

wrapper_value_type!(BoolValue: IntValue);

wrapper_value_type!(IntValue: IntValue);

wrapper_value_type!(FloatValue: FloatValue);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PointerValue<'ctx> {
    pub(super) value: values::PointerValue<'ctx>,
    pub(super) val_type: TypeEnum<'ctx>,
}

impl<'ctx> Deref for PointerValue<'ctx> {
    type Target = values::PointerValue<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'ctx> PointerValue<'ctx> {
    pub(super) fn new(value: values::PointerValue<'ctx>, val_type: TypeEnum<'ctx>) -> Self {
        Self { value, val_type }
    }
}

macro_rules! value_enum {
    ($name:ident : $($type:ident),*) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum $name<'ctx> {
            $(
                $type($type<'ctx>),
            )*
        }
    };
}

value_enum!(ValueEnum : BoolValue, IntValue, FloatValue, PointerValue);
