use inkwell::builder::BuilderError;

#[derive(Debug)]
pub enum GenError {
    Call,
    InvalidFunctionDefinition,
    FunctionNotFound,
    TypeNotFound,
    InvalidType,
    IdentConflict,
    VariableNotFound,
    Build(BuilderError),
}

impl From<BuilderError> for GenError {
    fn from(err: BuilderError) -> Self {
        GenError::Build(err)
    }
}
