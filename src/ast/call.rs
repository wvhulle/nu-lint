use nu_protocol::{
    BlockId, Span,
    ast::{Call, Expr, Expression},
};

use super::{block::BlockExt, expression::ExpressionExt};
use crate::{ast::span::SpanExt, context::LintContext};

pub trait CallExt {
    fn get_call_name(&self, context: &LintContext) -> String;
    fn is_call_to_command(&self, command_name: &str, context: &LintContext) -> bool;
    fn get_first_positional_arg(&self) -> Option<&Expression>;
    fn get_positional_arg(&self, index: usize) -> Option<&Expression>;
    #[must_use]
    fn loop_var_from_each(&self, context: &LintContext) -> Option<String>;
    #[must_use]
    fn loop_var_from_for(&self, context: &LintContext) -> Option<String>;
    #[must_use]
    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)>;
    #[must_use]
    fn extract_function_definition(&self, context: &LintContext) -> Option<(BlockId, String)>;
    #[must_use]
    fn extract_variable_declaration(
        &self,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)>;
    fn get_else_branch(&self) -> Option<(bool, &Expression)>;
    fn has_no_else_branch(&self) -> bool;
    fn get_nested_single_if<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call>;
    fn generate_collapsed_if(&self, context: &LintContext) -> Option<String>;
    fn uses_variable(&self, var_id: nu_protocol::VarId) -> bool;
    fn is_filesystem_command(&self, context: &LintContext) -> bool;
    fn extract_print_message(&self, context: &LintContext) -> Option<String>;
    fn extract_exit_code(&self) -> Option<i64>;
    fn has_named_flag(&self, flag_name: &str) -> bool;
}

impl CallExt for Call {
    fn get_call_name(&self, context: &LintContext) -> String {
        context
            .working_set
            .get_decl(self.decl_id)
            .name()
            .to_string()
    }

    fn is_call_to_command(&self, command_name: &str, context: &LintContext) -> bool {
        self.get_call_name(context) == command_name
    }

    fn get_first_positional_arg(&self) -> Option<&Expression> {
        self.get_positional_arg(0)
    }

    fn get_positional_arg(&self, index: usize) -> Option<&Expression> {
        self.arguments.get(index).and_then(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr)
            | nu_protocol::ast::Argument::Unknown(expr) => Some(expr),
            _ => None,
        })
    }

    fn loop_var_from_each(&self, context: &LintContext) -> Option<String> {
        let first_arg = self.get_first_positional_arg()?;
        let block_id = first_arg.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let var_id = block.signature.required_positional.first()?.var_id?;

        let var = context.working_set.get_variable(var_id);
        Some(var.declaration_span.text(context).to_string())
    }

    fn loop_var_from_for(&self, context: &LintContext) -> Option<String> {
        let var_arg = self.get_first_positional_arg()?;
        var_arg.extract_variable_name(context)
    }

    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)> {
        let name_arg = self.get_first_positional_arg()?;
        let name = name_arg.span.text(context);
        Some((name.to_string(), name_arg.span))
    }

    fn extract_function_definition(&self, context: &LintContext) -> Option<(BlockId, String)> {
        let decl_name = self.get_call_name(context);
        if !matches!(decl_name.as_str(), "def" | "export def") {
            return None;
        }

        let name_arg = self.get_first_positional_arg()?;
        let name = name_arg.span.text(context);

        let body_expr = self.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        Some((block_id, name.to_string()))
    }

    fn extract_variable_declaration(
        &self,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)> {
        let decl_name = self.get_call_name(context);
        if !matches!(decl_name.as_str(), "let" | "mut") {
            return None;
        }

        let var_arg = self.get_first_positional_arg()?;

        if let Expr::VarDecl(var_id) = &var_arg.expr {
            let var_name = var_arg.span.text(context);
            Some((*var_id, var_name.to_string(), var_arg.span))
        } else {
            None
        }
    }

    fn get_else_branch(&self) -> Option<(bool, &Expression)> {
        let else_arg = self.get_positional_arg(2)?;

        match &else_arg.expr {
            Expr::Keyword(keyword) => match &keyword.expr.expr {
                Expr::Call(_) => Some((true, &keyword.expr)),
                Expr::Block(_) => Some((false, &keyword.expr)),
                _ => None,
            },
            Expr::Block(_) => Some((false, else_arg)),
            _ => None,
        }
    }

    fn has_no_else_branch(&self) -> bool {
        self.get_else_branch().is_none()
    }

    fn get_nested_single_if<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call> {
        let then_block = self.get_positional_arg(1)?;
        let then_block_id = then_block.extract_block_id()?;
        then_block_id.get_single_if_call(context)
    }

    fn generate_collapsed_if(&self, context: &LintContext) -> Option<String> {
        self.has_no_else_branch().then_some(())?;

        let inner_call = self.get_nested_single_if(context)?;

        inner_call.has_no_else_branch().then_some(())?;

        let outer_condition = self.get_first_positional_arg()?;
        let inner_condition = inner_call.get_first_positional_arg()?;
        let inner_body = inner_call.get_positional_arg(1)?;

        let outer_cond = outer_condition.span_text(context).trim();
        let inner_cond = inner_condition.span_text(context).trim();
        let body = inner_body.span_text(context).trim();

        Some(format!("if {outer_cond} and {inner_cond} {body}"))
    }

    fn uses_variable(&self, var_id: nu_protocol::VarId) -> bool {
        self.arguments.iter().any(|arg| match arg {
            nu_protocol::ast::Argument::Positional(expr)
            | nu_protocol::ast::Argument::Unknown(expr)
            | nu_protocol::ast::Argument::Named((_, _, Some(expr))) => expr.matches_var(var_id),
            _ => false,
        })
    }

    fn is_filesystem_command(&self, context: &LintContext) -> bool {
        use nu_protocol::Category;

        let decl = context.working_set.get_decl(self.decl_id);
        let signature = decl.signature();
        matches!(signature.category, Category::FileSystem | Category::Path)
    }

    fn extract_print_message(&self, context: &LintContext) -> Option<String> {
        self.get_first_positional_arg()
            .map(|expr| expr.span_text(context).to_string())
    }

    fn extract_exit_code(&self) -> Option<i64> {
        self.get_first_positional_arg()
            .and_then(|code_expr| match &code_expr.expr {
                Expr::Int(code) => Some(*code),
                _ => None,
            })
    }

    fn has_named_flag(&self, flag_name: &str) -> bool {
        self.arguments.iter().any(|arg| {
            matches!(
                arg,
                nu_protocol::ast::Argument::Named(named) if named.0.item == flag_name
            )
        })
    }
}
