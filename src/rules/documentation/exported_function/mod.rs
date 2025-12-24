use nu_protocol::ast::{Call, Expr};

use crate::{
    LintLevel, ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation,
};

/// Check if there's a documentation comment before the given span
/// Since comments are not preserved in AST, we need to check source text
fn has_doc_comment_before(context: &LintContext, span: nu_protocol::Span) -> bool {
    let before_text = context.source_before_span(span);
    let lines: Vec<&str> = before_text.lines().collect();

    if lines.is_empty() {
        return false;
    }

    // Look for documentation comments, skipping over attributes and empty lines
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip attribute lines (like @example, @search-terms, etc.)
        if trimmed.starts_with('@') {
            continue;
        }

        // Check if it's a documentation comment (not a shebang ##)
        let is_comment = trimmed.starts_with('#') && !trimmed.starts_with("##");
        return is_comment;
    }

    false
}

fn check_exported_function(call: &Call, context: &LintContext) -> Option<Violation> {
    let decl_name = call.get_call_name(context);

    if decl_name != "export def" {
        return None;
    }

    let (func_name, name_span) = call.extract_declaration_name(context)?;

    let has_docs = has_doc_comment_before(context, call.head);

    if has_docs {
        None
    } else {
        Some(
            Violation::new(
                format!("Exported function '{func_name}' lacks documentation comment"),
                call.head,
            )
            .with_primary_label("missing doc comment")
            .with_extra_label("exported function", name_span)
            .with_help(format!(
                "Add a documentation comment (starting with #) above the export.\nExample:\n  # \
                 Description of what this function does\n  export def {func_name} [] {{ ... }}"
            )),
        )
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            check_exported_function(call, ctx).into_iter().collect()
        } else {
            vec![]
        }
    })
}

pub const RULE: Rule = Rule::new(
    "add_doc_comment_exported_fn",
    "Exported functions should have documentation comments",
    check,
    LintLevel::Hint,
)
.with_doc_url("https://www.nushell.sh/book/custom_commands.html#documenting-your-command");

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
