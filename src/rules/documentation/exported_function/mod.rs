use nu_protocol::ast::{Call, Expr};

use crate::{ast::call::CallExt, context::LintContext, rule::Rule, violation::Violation};

/// Check if there's a documentation comment before the given span
/// Since comments are not preserved in AST, we need to check source text
fn has_doc_comment_before(context: &LintContext, span: nu_protocol::Span) -> bool {
    let before_text = &context.source[..span.start];
    let lines: Vec<&str> = before_text.lines().collect();

    if lines.is_empty() {
        return false;
    }

    // Look for the last non-empty line before the span
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // Skip empty lines
        }

        // Check if it's a documentation comment
        let is_comment = trimmed.starts_with('#') && !trimmed.starts_with("##");

        let is_test_comment = trimmed.to_lowercase().contains("bad:")
            || trimmed.to_lowercase().contains("good:")
            || trimmed.to_lowercase().contains("todo:")
            || trimmed.to_lowercase().contains("fixme:")
            || trimmed.to_lowercase().contains("test:")
            || trimmed.to_lowercase().contains("example:");

        return is_comment && !is_test_comment;
    }

    false
}

fn check_exported_function(call: &Call, context: &LintContext) -> Option<Violation> {
    let decl_name = call.get_call_name(context);

    if decl_name != "export def" {
        return None;
    }

    let (func_name, _name_span) = call.extract_declaration_name(context)?;

    let has_docs = has_doc_comment_before(context, call.head);

    if has_docs {
        None
    } else {
        Some(
            Violation::new(format!("Exported function '{func_name}' is missing documentation"),
                call.head,
            )
            .with_help(format!(
                "Add a documentation comment above the function:\n# Description of \
                 {func_name}\nexport def {func_name} ..."
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

pub const fn rule() -> Rule {
    Rule::new(
        "exported_function_docs",
        "Exported functions should have documentation comments",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/modules.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
