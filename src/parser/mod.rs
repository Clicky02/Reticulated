use crate::{
    anyhow_ext::AnyhowResultExt,
    lexer::{KeywordKind, LiteralKind, OperatorKind, ReadTokens, Token, TokenKind},
};

use anyhow::{anyhow, Result};

use core::panic;

mod expression;
mod statement;

pub use expression::*;
pub use statement::*;

/// Parses tokens into an AST.
///
/// The parser relies on the underlying tokens provider to output an EOF token as the last token.
/// If the last token is not an EOF, it will likely panic.
pub struct Parser<R: ReadTokens> {
    tokens: R,
}

impl<R: ReadTokens> Parser<R> {
    pub fn new(tokens: R) -> Self {
        Parser { tokens }
    }

    /// Parses the tokens into an AST.
    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn is_at_end(&mut self) -> bool {
        let next = self.tokens.try_peek_next();
        match next {
            Some(&Token {
                kind: TokenKind::EOF,
                ..
            }) => true,
            None => true,
            _ => false,
        }
    }

    fn statement(&mut self) -> Result<Statement> {
        // statement -> (declaration | assignment | function_declaration | extern_function
        // | if_statement | return_statement | expression) "\n"

        let next = self.tokens.peek_next();
        let pos = next.span.start;
        let statement = match next.kind {
            TokenKind::Identifier(_) => match self.tokens.peek(1).kind {
                TokenKind::Colon => self.declaration().parsing_ctx("declaration", pos)?,
                TokenKind::Operator(op) if op.is_assign_op() => {
                    self.assignment().parsing_ctx("assignment", pos)?
                }
                _ => self.expr_statement().parsing_ctx("expression", pos)?,
            },
            TokenKind::Keyword(KeywordKind::Def) => Statement::FunctionDeclaration(
                self.fn_declaration()
                    .parsing_ctx("function declaration", pos)?,
            ),
            TokenKind::Keyword(KeywordKind::Extern) => self
                .extern_fn_declaration()
                .parsing_ctx("function declaration", pos)?,
            TokenKind::Keyword(KeywordKind::If) => {
                self.if_statement().parsing_ctx("if statement", pos)?
            }
            TokenKind::Keyword(KeywordKind::Return) => self
                .return_statement()
                .parsing_ctx("return statement", pos)?,
            TokenKind::Keyword(KeywordKind::Struct) => {
                self.struct_definition().parsing_ctx("struct", pos)?
            }
            TokenKind::Keyword(KeywordKind::While) => {
                self.while_loop().parsing_ctx("while loop", pos)?
            }
            _ => self.expr_statement().parsing_ctx("expression", pos)?,
        };

        // TODO: Newline?

        Ok(statement)
    }

    fn expr_statement(&mut self) -> Result<Statement> {
        // expr_statement -> expression
        // assignment -> access "=" expression

        let expr = self.expression()?;
        let assign_op = self.match_assign_op();

        if let Some((token, op)) = assign_op {
            if let Expression::Access(expr, ident) = expr {
                let rvalue = self.expression()?;
                Ok(Statement::Assignment {
                    lvalue: LValue::Access(expr, ident),
                    op,
                    expression: rvalue,
                })
            } else {
                Err(anyhow!(
                    "Expected identifier or access at {}, found expression ({:?}) instead.",
                    token.span,
                    expr
                ))
            }
        } else {
            Ok(Statement::Expression(expr))
        }
    }

    fn declaration(&mut self) -> Result<Statement> {
        // declaration -> IDENTIFIER ":" IDENTIFIER "=" expression

        let identifier = self.tokens.expect_identifier()?;

        self.tokens.expect(TokenKind::Colon)?; // Skip the colon

        let type_identifier = self.tokens.expect_identifier()?;

        self.tokens.expect_operator(OperatorKind::Assign)?; // Skip the equals sign

        let expression = self.expression()?;

        Ok(Statement::Declaration {
            identifier,
            type_identifier,
            expression,
        })
    }

    fn assignment(&mut self) -> Result<Statement> {
        // assignment -> IDENTIFIER "=" expression

        let identifier = self.tokens.expect_identifier()?;
        let (_, op) = self.match_assign_op().unwrap();
        let expression = self.expression()?;

        Ok(Statement::Assignment {
            lvalue: LValue::Ident(identifier),
            op,
            expression,
        })
    }

    fn fn_declaration(&mut self) -> Result<FuncDeclaration> {
        // function_declaration -> "fn" IDENTIFIER "(" parameters ")" "->" IDENTIFIER block

        self.tokens.expect_keyword(KeywordKind::Def)?;

        let identifier = self.tokens.expect_identifier()?;

        self.tokens.expect(TokenKind::OpenParenthesis)?;

        let (takes_self, parameters) = self.fn_parameters()?;

        self.tokens.expect(TokenKind::CloseParenthesis)?;
        self.tokens.expect(TokenKind::Arrow)?;

        let return_identifier = self.tokens.expect_identifier()?;

        let body = self.block()?;

        Ok(FuncDeclaration::new(
            identifier,
            takes_self,
            parameters,
            return_identifier,
            body,
        ))
    }

    fn extern_fn_declaration(&mut self) -> Result<Statement> {
        // extern_function -> "extern" "def" IDENTIFIER "(" extern_parameters ")" "->" IDENTIFIER

        self.tokens.expect_keyword(KeywordKind::Extern)?;
        self.tokens.expect_keyword(KeywordKind::Def)?;

        let identifier = self.tokens.expect_identifier()?;

        self.tokens.expect(TokenKind::OpenParenthesis)?;

        let (has_self, parameters) = self.fn_parameters()?;
        if has_self {
            return Err(anyhow!(
                "Extern functions cannot have 'self' as a parameter."
            ));
        }

        self.tokens.expect(TokenKind::CloseParenthesis)?;
        self.tokens.expect(TokenKind::Arrow)?;

        let return_identifier = self.tokens.expect_identifier()?;

        Ok(Statement::ExternFunctionDeclaration {
            identifier,
            parameters,
            return_identifier,
        })
    }

    fn fn_parameters(&mut self) -> Result<(bool, Vec<FuncParameter>)> {
        // parameters -> ("self" ",")? (parameter ("," "*"? parameter)*)?
        // parameter -> IDENTIFIER ":" IDENTIFIER

        let mut params = Vec::new();
        let mut is_first = true;
        let has_self = if self.tokens.check(TokenKind::Keyword(KeywordKind::Self_)) {
            self.tokens.advance();
            is_first = false;
            true
        } else {
            false
        };

        while !self.tokens.check(TokenKind::CloseParenthesis) {
            if !is_first {
                self.tokens.expect(TokenKind::Comma)?;
            } else {
                is_first = false;
            }

            let var_args = if self
                .tokens
                .check(TokenKind::Operator(OperatorKind::Multiply))
            {
                self.tokens.advance();
                true
            } else {
                false
            };

            let identifier = self.tokens.expect_identifier()?;
            self.tokens.expect(TokenKind::Colon)?;
            let type_identifier = self.tokens.expect_identifier()?;
            params.push(FuncParameter::new(identifier, type_identifier, var_args));
        }

        Ok((has_self, params))
    }

    fn if_statement(&mut self) -> Result<Statement> {
        // if_statement -> "if" expression block ("else" "if" expression block)* ("else" block)?

        self.tokens.expect_keyword(KeywordKind::If)?;

        let condition = self.expression()?;
        let then_branch = self.block()?;

        let mut else_if_branches = Vec::new();
        let mut else_branch = None;

        while self.tokens.check(TokenKind::Keyword(KeywordKind::Else)) {
            self.tokens.advance();

            match self.tokens.peek_next().kind {
                TokenKind::Keyword(KeywordKind::If) => {
                    self.tokens.expect_keyword(KeywordKind::If)?;
                    let condition = self.expression()?;
                    let branch = self.block()?;
                    else_if_branches.push((condition, branch));
                }
                TokenKind::OpenBrace => {
                    let branch = self.block()?;
                    else_branch = Some(branch);
                }
                _ => panic!("Expected 'if' or '{{' after 'else'"),
            }
        }

        Ok(Statement::IfStatement {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        })
    }

    fn return_statement(&mut self) -> Result<Statement> {
        // return_statement -> "return" expression

        self.tokens.expect_keyword(KeywordKind::Return)?;
        let expression = self.expression()?;

        Ok(Statement::ReturnStatement { expression })
    }

    fn struct_definition(&mut self) -> Result<Statement> {
        // struct_declaration -> "struct" IDENTIFIER "{" (struct_field",")* "}"
        // struct_field -> IDENTIFIER: IDENTIFIER

        self.tokens.expect_keyword(KeywordKind::Struct)?;

        let identifier = self.tokens.expect_identifier()?;

        self.tokens.expect(TokenKind::OpenBrace)?;

        let mut fields = Vec::new();
        let mut fns = Vec::new();

        while !self.tokens.check(TokenKind::CloseBrace) {
            let next = self.tokens.peek_next();
            let pos = next.span.start;
            match &next.kind {
                TokenKind::Keyword(KeywordKind::Def) => {
                    let next_fn = self
                        .fn_declaration()
                        .parsing_ctx("function declaration", pos)?;
                    if !next_fn.takes_self {
                        return Err(anyhow!(
                            "Struct methods must take 'self' as the first parameter."
                        ));
                    }
                    fns.push(next_fn);
                }
                TokenKind::Identifier(_) => {
                    let field_name = self.tokens.expect_identifier()?;
                    self.tokens.expect(TokenKind::Colon)?; // Skip the colon

                    let field_type = self.tokens.expect_identifier()?;
                    fields.push((field_name, field_type));

                    self.tokens.expect(TokenKind::Comma)?;
                }
                _ => {
                    return Err(anyhow!(
                        "Expected identifier or function declaration at {} found {}",
                        next.span.start,
                        next.kind
                    ))
                }
            }
        }

        self.tokens.expect(TokenKind::CloseBrace)?;

        Ok(Statement::StructDefinition {
            identifier,
            fields,
            fns,
        })
    }

    fn while_loop(&mut self) -> Result<Statement> {
        // while_loop -> "while" expression block

        self.tokens.expect_keyword(KeywordKind::While)?;

        let condition = self.expression()?;

        let body = self.block()?;

        Ok(Statement::WhileLoop {
            condition,
            block: body,
        })
    }

    fn block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        self.tokens.expect(TokenKind::OpenBrace)?;

        while !self.tokens.check(TokenKind::CloseBrace) {
            statements.push(self.statement()?);
        }

        self.tokens.advance(); // Eat the close brace

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expression> {
        self.logical()
    }

    fn logical(&mut self) -> Result<Expression> {
        // logical -> equality ( ("or" | "and") equality )*

        let mut expr = self.equality()?;

        while let Some(op) = self.match_logical_op() {
            let right = self.equality()?;
            expr = Expression::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_equality_op() {
            let right = self.comparison()?;
            expr = Expression::BinaryFn(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_comparison_op() {
            let right = self.term()?;
            expr = Expression::BinaryFn(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_term_op() {
            let right = self.factor()?;
            expr = Expression::BinaryFn(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_factor_op() {
            let right = self.unary()?;
            expr = Expression::BinaryFn(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression> {
        // unary -> ( "!" | "-" ) unary | invoke

        if let Some(op) = self.match_unary_fn_op() {
            let right = self.unary()?;
            return Ok(Expression::UnaryFn(op, Box::new(right)));
        }

        if let Some(op) = self.match_unary_op() {
            let right = self.unary()?;
            return Ok(Expression::Unary(op, Box::new(right)));
        }

        self.invoke()
    }

    fn invoke(&mut self) -> Result<Expression> {
        // invoke -> (invoke | access)  "(" parameter_values ")"

        let mut expr = self.access()?;

        while self.tokens.check(TokenKind::OpenParenthesis) {
            // allow multiple invokations in a row
            self.tokens.advance(); // Eat open paranthesis

            let mut args = Vec::new();

            let mut first = true;
            while self.tokens.peek_next().kind != TokenKind::CloseParenthesis {
                if !first {
                    self.tokens.expect(TokenKind::Comma)?;
                }

                args.push(self.expression()?);

                first = false;
            }

            self.tokens.expect(TokenKind::CloseParenthesis)?;

            expr = Expression::Invoke(Box::new(expr), args)
        }

        Ok(expr)
    }

    fn access(&mut self) -> Result<Expression> {
        // access -> (access | primary) "." IDENTIFIER

        let mut expr = self.primary()?;

        while self.tokens.check(TokenKind::Period) {
            self.tokens.advance(); // eat the period
            let member = self.tokens.expect_identifier()?;
            expr = Expression::Access(Box::new(expr), member);
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expression> {
        // primary -> IDENTIFIER | LITERAL | "(" expression ")"

        let Some(next) = self.tokens.advance() else {
            return Err(anyhow!("Unexpectedly reached end of input."));
        };

        match next.kind {
            TokenKind::Keyword(KeywordKind::Self_) => {
                Ok(Expression::Primary(Primary::Identifier("self".to_string())))
            }
            TokenKind::Literal(LiteralKind::Integer(value)) => {
                Ok(Expression::Primary(Primary::Integer(value)))
            }
            TokenKind::Literal(LiteralKind::Float(value)) => {
                Ok(Expression::Primary(Primary::Float(value)))
            }
            TokenKind::Literal(LiteralKind::String(value)) => {
                Ok(Expression::Primary(Primary::String(value)))
            }
            TokenKind::Literal(LiteralKind::Boolean(value)) => {
                Ok(Expression::Primary(Primary::Bool(value)))
            }
            TokenKind::Identifier(identifier) => {
                Ok(Expression::Primary(Primary::Identifier(identifier)))
            }
            // TODO: Add null/none type
            TokenKind::OpenParenthesis => {
                let expr = self.expression()?;
                self.tokens.expect(TokenKind::CloseParenthesis)?;
                Ok(Expression::Primary(Primary::Grouping(Box::new(expr))))
            }
            _ => Err(anyhow!(
                "Expected primary expression at {} found {}",
                next.span.start,
                next.kind
            )),
        }
    }

    fn match_logical_op(&mut self) -> Option<BinaryOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::And) => {
                self.tokens.advance();
                Some(BinaryOp::And)
            }
            TokenKind::Operator(OperatorKind::Or) => {
                self.tokens.advance();
                Some(BinaryOp::Or)
            }
            _ => None,
        }
    }

    fn match_assign_op(&mut self) -> Option<(Token, AssignOp)> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::Assign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::Assign))
            }
            TokenKind::Operator(OperatorKind::AddAssign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::AddAssign))
            }
            TokenKind::Operator(OperatorKind::SubtractAssign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::SubtractAssign))
            }
            TokenKind::Operator(OperatorKind::MultiplyAssign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::MultiplyAssign))
            }
            TokenKind::Operator(OperatorKind::DivideAssign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::DivideAssign))
            }
            TokenKind::Operator(OperatorKind::ModuloAssign) => {
                Some((self.tokens.advance().unwrap(), AssignOp::ModuloAssign))
            }
            _ => None,
        }
    }

    fn match_equality_op(&mut self) -> Option<BinaryFnOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::NotEqual) => {
                self.tokens.advance();
                Some(BinaryFnOp::NotEqual)
            }
            TokenKind::Operator(OperatorKind::Equal) => {
                self.tokens.advance();
                Some(BinaryFnOp::Equal)
            }
            _ => None,
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryFnOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::GreaterThan) => {
                self.tokens.advance();
                Some(BinaryFnOp::Greater)
            }
            TokenKind::Operator(OperatorKind::GreaterThanOrEqual) => {
                self.tokens.advance();
                Some(BinaryFnOp::GreaterEqual)
            }
            TokenKind::Operator(OperatorKind::LessThan) => {
                self.tokens.advance();
                Some(BinaryFnOp::Less)
            }
            TokenKind::Operator(OperatorKind::LessThanOrEqual) => {
                self.tokens.advance();
                Some(BinaryFnOp::LessEqual)
            }
            _ => None,
        }
    }

    fn match_term_op(&mut self) -> Option<BinaryFnOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::Add) => {
                self.tokens.advance();
                Some(BinaryFnOp::Add)
            }
            TokenKind::Operator(OperatorKind::Subtract) => {
                self.tokens.advance();
                Some(BinaryFnOp::Subtract)
            }
            _ => None,
        }
    }

    fn match_factor_op(&mut self) -> Option<BinaryFnOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::Multiply) => {
                self.tokens.advance();
                Some(BinaryFnOp::Multiply)
            }
            TokenKind::Operator(OperatorKind::Divide) => {
                self.tokens.advance();
                Some(BinaryFnOp::Divide)
            }
            TokenKind::Operator(OperatorKind::Modulo) => {
                self.tokens.advance();
                Some(BinaryFnOp::Modulo)
            }
            TokenKind::Operator(OperatorKind::Exponentiate) => {
                self.tokens.advance();
                Some(BinaryFnOp::Exponentiate)
            }
            TokenKind::Operator(OperatorKind::MatMul) => {
                self.tokens.advance();
                Some(BinaryFnOp::MatMul)
            }
            _ => None,
        }
    }

    fn match_unary_fn_op(&mut self) -> Option<UnaryFnOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::Subtract) => {
                self.tokens.advance();
                Some(UnaryFnOp::Negate)
            }
            _ => None,
        }
    }

    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        match self.tokens.peek_next().kind {
            TokenKind::Operator(OperatorKind::Not) => {
                self.tokens.advance();
                Some(UnaryOp::Not)
            }
            _ => None,
        }
    }
}
