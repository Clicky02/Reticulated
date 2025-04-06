use super::Expression;

#[derive(Debug)]
pub enum LValue {
    Ident(String),
    Access(Box<Expression>, String),
}

#[derive(Debug)]
pub enum Statement {
    Declaration {
        identifier: String,
        type_identifier: String,
        expression: Expression,
    },
    Assignment {
        lvalue: LValue,
        expression: Expression,
    },
    FunctionDeclaration(FuncDeclaration),
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
    StructDefinition {
        identifier: String,
        fields: Vec<(String, String)>,
        fns: Vec<FuncDeclaration>,
    },

    WhileLoop {
        condition: Expression,
        block: Vec<Statement>,
    },
}

#[derive(Debug)]
pub struct FuncDeclaration {
    pub identifier: String,
    pub takes_self: bool,
    pub params: Vec<FuncParameter>,
    pub return_identifier: String,
    pub body: Vec<Statement>,
}

impl FuncDeclaration {
    pub fn new(
        identifier: String,
        takes_self: bool,
        parameters: Vec<FuncParameter>,
        return_identifier: String,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            identifier,
            takes_self,
            params: parameters,
            return_identifier,
            body,
        }
    }
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
