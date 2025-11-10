use nu_protocol::{
    BlockId, Span,
    ast::{Argument, Call, Expr, Expression},
};

use super::{block::BlockExt, expression::ExpressionExt};
use crate::{ast::span::SpanExt, context::LintContext};

pub trait CallExt {
    /// Gets the command name of this call. Example: `ls -la` returns "ls"
    fn get_call_name(&self, context: &LintContext) -> String;
    /// Checks if call is to a specific command. Example: `if $x { }` matches
    /// "if"
    fn is_call_to_command(&self, command_name: &str, context: &LintContext) -> bool;
    /// Gets first positional argument. Example: `ls /tmp` returns `/tmp`
    fn get_first_positional_arg(&self) -> Option<&Expression>;
    /// Gets positional argument at index. Example: `parse "{x} {y}"` at index 0
    /// returns pattern
    fn get_positional_arg(&self, index: usize) -> Option<&Expression>;
    #[must_use]
    /// Extracts loop variable from each closure. Example: `each { |item| ... }`
    /// returns "item"
    fn loop_var_from_each(&self, context: &LintContext) -> Option<String>;
    #[must_use]
    /// Extracts loop variable from for loop. Example: `for item in $list { }`
    /// returns "item"
    fn loop_var_from_for(&self, context: &LintContext) -> Option<String>;
    #[must_use]
    /// Extracts declaration name and span. Example: `def foo [] { }` returns
    /// ("foo", span)
    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)>;
    #[must_use]
    /// Extracts function definition block and name. Example: `def process [] {
    /// ls }` returns block and "process"
    fn extract_function_definition(&self, context: &LintContext) -> Option<(BlockId, String)>;
    #[must_use]
    /// Extracts variable declaration. Example: `let x = 5` returns `(var_id,
    /// "x", span)`
    fn extract_variable_declaration(
        &self,
        context: &LintContext,
    ) -> Option<(nu_protocol::VarId, String, Span)>;
    /// Gets else branch from if call. Example: `if $x { } else { }` returns
    /// else block
    fn get_else_branch(&self) -> Option<(bool, &Expression)>;
    /// Checks if if call has no else branch. Example: `if $x { 1 }` returns
    /// true
    fn has_no_else_branch(&self) -> bool;
    /// Gets nested single if call from then branch. Example: `if $x { if $y { }
    /// }`
    fn get_nested_single_if<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call>;
    /// Generates collapsed if condition text. Example: `if $x { if $y { } }`
    /// becomes `if $x and $y { }`
    fn generate_collapsed_if(&self, context: &LintContext) -> Option<String>;
    /// Checks if call uses a variable. Example: `print $msg` uses `$msg`
    fn uses_variable(&self, var_id: nu_protocol::VarId) -> bool;
    /// Checks if call is a filesystem command. Example: `mkdir`, `cd`, or `rm`
    fn is_filesystem_command(&self, context: &LintContext) -> bool;
    /// Extracts message from print call. Example: `print "Error: failed"`
    /// returns "Error: failed"
    fn extract_print_message(&self, context: &LintContext) -> Option<String>;
    /// Extracts exit code from exit call. Example: `exit 1` returns 1
    fn extract_exit_code(&self) -> Option<i64>;
    /// Checks if call has a named flag. Example: `ls --all` has flag "all"
    fn has_named_flag(&self, flag_name: &str) -> bool;
    /// Extracts iterator expression from for loop call. Example: `for x in
    /// $list { }` returns `$list`
    fn get_for_loop_iterator(&self) -> Option<&Expression>;
    /// Extracts body block from for loop call. Example: `for x in $list { ...
    /// }` returns body block
    fn get_for_loop_body(&self) -> Option<nu_protocol::BlockId>;
    /// Gets named argument expression by flag name. Example: `try { ... }
    /// --catch { ... }` returns catch block
    fn get_named_arg_expr(&self, flag_name: &str) -> Option<&Expression>;
    /// Checks if this is a control flow command. Example: `if`, `for`,
    /// `while`, `match`, `try`
    fn is_control_flow_command(&self, context: &LintContext) -> bool;
    /// Gets all argument expressions from a call. Example: positional, named,
    /// spread arguments
    fn all_arg_expressions(&self) -> Vec<&Expression>;
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
            Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
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
        let name = match &name_arg.expr {
            Expr::String(s) | Expr::RawString(s) => s.clone(),
            _ => name_arg.span.text(context).to_string(),
        };

        let body_expr = self.get_positional_arg(2)?;
        let block_id = body_expr.extract_block_id()?;

        Some((block_id, name))
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
            Argument::Positional(expr)
            | Argument::Unknown(expr)
            | Argument::Named((_, _, Some(expr))) => expr.matches_var(var_id),
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
                Argument::Named(named) if named.0.item == flag_name
            )
        })
    }

    fn get_for_loop_iterator(&self) -> Option<&Expression> {
        let iter_arg = self.arguments.get(1)?;
        match iter_arg {
            Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
            _ => None,
        }
    }

    fn get_for_loop_body(&self) -> Option<nu_protocol::BlockId> {
        let block_arg = self.arguments.last()?;
        let (Argument::Positional(block_expr) | Argument::Unknown(block_expr)) = block_arg else {
            return None;
        };

        match &block_expr.expr {
            Expr::Block(block_id) => Some(*block_id),
            _ => None,
        }
    }

    fn get_named_arg_expr(&self, flag_name: &str) -> Option<&Expression> {
        self.arguments.iter().find_map(|arg| {
            if let Argument::Named(named) = arg
                && named.0.item == flag_name
            {
                named.2.as_ref()
            } else {
                None
            }
        })
    }

    fn is_control_flow_command(&self, context: &LintContext) -> bool {
        matches!(
            self.get_call_name(context).as_str(),
            "if" | "for" | "while" | "match" | "try"
        )
    }

    fn all_arg_expressions(&self) -> Vec<&Expression> {
        self.arguments
            .iter()
            .filter_map(|arg| match arg {
                Argument::Positional(e) | Argument::Unknown(e) | Argument::Spread(e) => Some(e),
                Argument::Named(named) => named.2.as_ref(),
            })
            .collect()
    }
}
