use crate::{
    codegen::env::id::BOOL_ID,
    parser::{Expression, Statement},
};

use super::{env::Environment, err::GenError, CodeGen};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn compile_if_statement(
        &mut self,
        condition: &Expression,
        then_branch: &Vec<Statement>,
        else_if_branches: &Vec<(Expression, Vec<Statement>)>,
        else_branch: &Option<Vec<Statement>>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        // TODO: Decrement Condition Expressions

        let (cond_ptr, cond_type_id) = self.compile_expression(condition, env)?;
        let cond_type = env.get_type(cond_type_id);

        assert_eq!(cond_type_id, BOOL_ID); // TODO: GenError

        let mut cond_val = self.extract_primitive(cond_ptr, cond_type.ink())?;
        self.free_pointer(cond_ptr, cond_type_id, env)?;

        let mut source_block = self.builder.get_insert_block().unwrap();
        let func = source_block.get_parent().unwrap();

        let mut then_block = self.ctx.append_basic_block(func, "then");
        let merge_block = self.ctx.append_basic_block(func, "merge");

        // Primary/Then branch
        self.builder.position_at_end(then_block);
        self.compile_block(&then_branch, env)?;
        self.builder.build_unconditional_branch(merge_block)?;

        // Else If branches
        for (condition, branch) in else_if_branches {
            // Create new branches
            let next_source_block = self
                .ctx
                .insert_basic_block_after(then_block, "elseifcondition");
            let next_then_block = self
                .ctx
                .insert_basic_block_after(next_source_block, "elseif");

            // Create branch from previous block to this new block
            self.builder.position_at_end(source_block);
            self.builder.build_conditional_branch(
                cond_val.into_int_value(),
                then_block,
                next_source_block,
            )?;

            // Compile condition
            self.builder.position_at_end(next_source_block);
            let (cond_ptr, cond_type_id) = self.compile_expression(condition, env)?;
            let cond_type = env.get_type(cond_type_id);
            // TODO: Check boolean?

            cond_val = self.extract_primitive(cond_ptr, cond_type.ink())?;
            self.free_pointer(cond_ptr, cond_type_id, env)?;

            source_block = next_source_block;
            then_block = next_then_block;

            // Compile branch statements
            self.builder.position_at_end(then_block);
            self.compile_block(&branch, env)?;
            self.builder.build_unconditional_branch(merge_block)?;
        }

        // Else
        if let Some(else_branch) = else_branch {
            // Create else branch
            let else_block = self.ctx.insert_basic_block_after(then_block, "else");

            // Create branch from previous block to this new block
            self.builder.position_at_end(source_block);
            self.builder.build_conditional_branch(
                cond_val.into_int_value(),
                then_block,
                else_block,
            )?;

            self.builder.position_at_end(else_block);
            self.compile_block(&else_branch, env)?;
            self.builder.build_unconditional_branch(merge_block)?;
        } else {
            // Create branch from previous block to outside the if statement
            self.builder.position_at_end(source_block);
            self.builder.build_conditional_branch(
                cond_val.into_int_value(),
                then_block,
                merge_block,
            )?;
        }

        self.builder.position_at_end(merge_block);

        Ok(())
    }

    pub(super) fn compile_while_loop(
        &mut self,
        condition: &Expression,
        block: &Vec<Statement>,
        env: &mut Environment<'ctx>,
    ) -> Result<(), GenError> {
        let condition_block = self
            .ctx
            .insert_basic_block_after(self.builder.get_insert_block().unwrap(), "condition");

        let body_block = self.ctx.insert_basic_block_after(condition_block, "body");
        let merge_block = self.ctx.insert_basic_block_after(body_block, "continue");

        self.builder.build_unconditional_branch(condition_block)?;
        self.builder.position_at_end(condition_block);

        let (expr_ptr, type_id) = self.compile_expression(condition, env)?;
        assert_eq!(type_id, BOOL_ID); // TODO Error

        let expr_type = env.get_type(type_id);
        let bool_val = self
            .extract_primitive(expr_ptr, expr_type.ink())?
            .into_int_value();
        self.free_pointer(expr_ptr, type_id, env)?;

        self.builder
            .build_conditional_branch(bool_val, body_block, merge_block)?;

        self.builder.position_at_end(body_block);
        self.compile_block(&block, env)?;
        self.builder.build_unconditional_branch(condition_block)?;

        self.builder.position_at_end(merge_block);

        Ok(())
    }
}
