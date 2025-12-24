use nu_protocol::{
    BlockId, Span,
    ast::{Argument, Call, Expr, Expression},
};

use super::{block::BlockExt, expression::ExpressionExt};
use crate::{ast::span::SpanExt, context::LintContext};

/// Checks if `actual_type` is compatible with `expected_type` for command
/// signature matching
fn is_type_compatible(expected: &nu_protocol::Type, actual: &nu_protocol::Type) -> bool {
    use nu_protocol::Type;

    match (expected, actual) {
        (e, a) if e == a => true,
        (Type::Any, _) | (_, Type::Any) => true,
        (Type::List(expected_inner), Type::List(actual_inner)) => {
            is_type_compatible(expected_inner, actual_inner)
        }
        _ => false,
    }
}

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
    /// Checks if this is a branching control flow command whose output type
    /// can be inferred from branch blocks. Example: `if`, `match`, `try`
    fn is_branching_control_flow(&self, context: &LintContext) -> bool;
    /// Checks if this is a control flow command that adds nesting depth.
    /// Example: `if`, `for`, `while`, `match`, `try`, `loop`
    fn is_control_flow_command(&self, context: &LintContext) -> bool;
    /// Gets all argument expressions from a call. Example: positional, named,
    /// spread arguments
    fn all_arg_expressions(&self) -> Vec<&Expression>;

    fn get_output_type(
        &self,
        context: &LintContext,
        pipeline_input: Option<nu_protocol::Type>,
    ) -> nu_protocol::Type;

    /// Infers unified output type from block arguments in control flow
    /// commands. Example: `if $x { "str" } else { "other" }` returns `string`
    fn infer_from_blocks(&self, context: &LintContext) -> Option<nu_protocol::Type>;
}

impl CallExt for Call {
    fn get_output_type(
        &self,
        context: &LintContext,
        pipeline_input: Option<nu_protocol::Type>,
    ) -> nu_protocol::Type {
        let decl = context.working_set.get_decl(self.decl_id);
        let sig = decl.signature();

        log::debug!(
            "get_output_type called for '{}': pipeline_input={pipeline_input:?}",
            self.get_call_name(context)
        );

        log::debug!(
            "Nu parser parsed output type for call '{}': {:?}",
            self.get_call_name(context),
            sig.get_output_type()
        );

        let has_pipeline_input = pipeline_input.is_some();
        let input_type = pipeline_input.unwrap_or_else(|| sig.get_input_type());
        log::debug!(
            "Final input_type used for call '{}': {:?} (from pipeline_input: {})",
            self.get_call_name(context),
            input_type,
            has_pipeline_input
        );

        log::debug!(
            "Command '{}' input_output_types: {:?}",
            self.get_call_name(context),
            sig.input_output_types
        );

        for (in_ty, out_ty) in &sig.input_output_types {
            if is_type_compatible(in_ty, &input_type) && !matches!(out_ty, nu_protocol::Type::Any) {
                log::debug!(
                    "Found compatible type mapping for '{}': {:?} -> {:?} (actual input: {:?})",
                    self.get_call_name(context),
                    in_ty,
                    out_ty,
                    input_type
                );
                return out_ty.clone();
            }
            log::debug!(
                "The signature with input type {:?} is not compatible with actual input type {:?} \
                 for command '{}'",
                in_ty,
                input_type,
                self.get_call_name(context)
            );
        }
        log::debug!(
            "Could not find compatible type mapping for '{}'",
            self.get_call_name(context)
        );

        if self.is_branching_control_flow(context)
            && let Some(inferred) = self.infer_from_blocks(context)
        {
            log::debug!(
                "Branching control flow '{}' inferred output type from blocks: {:?}",
                self.get_call_name(context),
                inferred
            );
            return inferred;
        }

        sig.get_output_type()
    }
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
        self.arguments
            .iter()
            .filter_map(|arg| match arg {
                Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
                _ => None,
            })
            .nth(index)
    }

    fn loop_var_from_each(&self, context: &LintContext) -> Option<String> {
        let first_arg = self.get_first_positional_arg()?;
        let block_id = first_arg.extract_block_id()?;

        let block = context.working_set.get_block(block_id);
        let var_id = block.signature.required_positional.first()?.var_id?;

        let var = context.working_set.get_variable(var_id);
        Some(var.declaration_span.source_code(context).to_string())
    }

    fn loop_var_from_for(&self, context: &LintContext) -> Option<String> {
        let var_arg = self.get_first_positional_arg()?;
        var_arg.extract_variable_name(context)
    }

    fn extract_declaration_name(&self, context: &LintContext) -> Option<(String, Span)> {
        let name_arg = self.get_first_positional_arg()?;
        let name = name_arg.span.source_code(context);
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
            _ => name_arg.span.source_code(context).to_string(),
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
            let var_name = var_arg.span.source_code(context);
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

    fn get_nested_single_if<'a>(&self, context: &'a LintContext<'a>) -> Option<&'a Call> {
        let then_block = self.get_positional_arg(1)?;
        let then_block_id = then_block.extract_block_id()?;
        context
            .working_set
            .get_block(then_block_id)
            .get_single_if_call(context)
    }

    fn generate_collapsed_if(&self, context: &LintContext) -> Option<String> {
        self.get_else_branch().is_none().then_some(())?;

        let inner_call = self.get_nested_single_if(context)?;

        inner_call.get_else_branch().is_none().then_some(())?;

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
        self.get_positional_arg(1)
    }

    fn get_for_loop_body(&self) -> Option<nu_protocol::BlockId> {
        self.arguments.last().and_then(|arg| match arg {
            Argument::Positional(expr) | Argument::Unknown(expr) => expr.extract_block_id(),
            _ => None,
        })
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

    fn is_branching_control_flow(&self, context: &LintContext) -> bool {
        matches!(self.get_call_name(context).as_str(), "if" | "match" | "try")
    }

    fn is_control_flow_command(&self, context: &LintContext) -> bool {
        matches!(
            self.get_call_name(context).as_str(),
            "if" | "for" | "while" | "loop" | "match" | "try"
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

    fn infer_from_blocks(&self, context: &LintContext) -> Option<nu_protocol::Type> {
        log::debug!("Inferring type from call with blocks");

        let mut block_types = self.positional_iter().filter_map(|arg| {
            arg.extract_block_id().map(|block_id| {
                let output = context
                    .working_set
                    .get_block(block_id)
                    .infer_output_type(context);
                log::debug!("Block {block_id:?} output type: {output:?}");
                output
            })
        });

        let first = block_types.next()?;
        log::debug!("First block type: {first:?}");

        let unified = block_types.try_fold(first, |acc, ty| {
            if acc == ty {
                Some(acc)
            } else {
                log::debug!("Block types differ: {acc:?} vs {ty:?}");
                None
            }
        })?;

        log::debug!("Unified block type: {unified:?}");
        Some(unified)
    }
}
