pub mod wrapper;

mod bool;
mod float;
mod function;
mod int;
mod pointer;

pub use bool::*;
pub use float::*;
pub use function::*;
use inkwell::types::BasicTypeEnum;
pub use int::*;
pub use pointer::*;

trait RetType<'ctx> {
    fn as_type_enum(&self) -> TypeEnum<'ctx>;
    fn try_as_inkwell_basic(&self) -> Result<BasicTypeEnum<'ctx>, String>;
}

macro_rules! type_enum {
    ($name:ident : $($type:ident),*) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum $name<'ctx> {
            $(
                $type($type<'ctx>),
            )*
        }

        $(
            impl<'ctx> From<$type<'ctx>> for $name<'ctx> {
                fn from(value: $type<'ctx>) -> Self {
                    $name::$type(value)
                }
            }
        )*
    };
}

type_enum!(TypeEnum : BoolType, IntType, FloatType, PointerType, FunctionType);

impl<'ctx> RetType<'ctx> for TypeEnum<'ctx> {
    fn as_type_enum(&self) -> TypeEnum<'ctx> {
        self.clone()
    }

    fn try_as_inkwell_basic(&self) -> Result<BasicTypeEnum<'ctx>, String> {
        match self {
            TypeEnum::BoolType(t) => t.try_as_inkwell_basic(),
            TypeEnum::IntType(t) => t.try_as_inkwell_basic(),
            TypeEnum::FloatType(t) => t.try_as_inkwell_basic(),
            TypeEnum::PointerType(t) => t.try_as_inkwell_basic(),
            TypeEnum::FunctionType(t) => t.try_as_inkwell_basic(),
        }
    }
}
