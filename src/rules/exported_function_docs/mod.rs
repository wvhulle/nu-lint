use nu_protocol::ast::Expr;

use crate::{
    ast_utils::{AstUtils, DeclarationUtils},
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

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

fn check_exported_function(call: &nu_protocol::ast::Call, context: &LintContext) -> Option<RuleViolation> {
    let decl_name = AstUtils::get_call_name(call, context);

    if decl_name != "export def" {
        return None;
    }

    let (func_name, _name_span) = DeclarationUtils::extract_declaration_name(call, context)?;

    let has_docs = has_doc_comment_before(context, call.head);

    if !has_docs {
        Some(
            RuleViolation::new_dynamic(
                "exported_function_docs",
                format!("Exported function '{func_name}' is missing documentation"),
                call.head,
            )
            .with_suggestion_dynamic(format!(
                "Add a documentation comment above the function:\n# Description of {func_name}\nexport def {func_name} ..."
            )),
        )
    } else {
        None
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| {
        if let Expr::Call(call) = &expr.expr {
            check_exported_function(call, ctx).into_iter().collect()
        } else {
            vec![]
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "exported_function_docs",
        RuleCategory::Documentation,
        Severity::Info,
        "Exported functions should have documentation comments",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
