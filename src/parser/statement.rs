use super::Expression;

#[derive(Debug)]
pub enum Statement {
    Declaration {
        identifier: String,
        type_identifier: String,
        expression: Expression,
    },
    Assignment {
        identifier: String,
        expression: Expression,
    },
    FunctionDeclaration {
        identifier: String,
        parameters: Vec<FuncParameter>,
        return_identifier: String,
        body: Vec<Statement>,
    },
    ExternFunctionDeclaration {
        identifier: String,
        parameters: Vec<FuncParameter>,
        return_identifier: String,
    },
    Expression(Expression),
    IfStatement {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_if_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    ReturnStatement {
        expression: Expression,
    },
}

#[derive(Debug)]
pub struct FuncParameter {
    pub identifier: String,
    pub type_identifier: String,
    pub var_args: bool,
}

impl FuncParameter {
    pub fn new(identifier: String, type_identifier: String, var_args: bool) -> Self {
        Self {
            identifier,
            type_identifier,
            var_args,
        }
    }
}
