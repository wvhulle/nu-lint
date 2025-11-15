use nu_protocol::ast::{Call, Expr, Expression, Traverse};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::RuleViolation,
};

const MAX_FUNCTION_LINE_LENGTH: usize = 80;

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::Call(call) = &expr.expr
                && let Some(value) = function_too_long(context, expr, call)
            {
                return vec![value];
            }
            vec![]
        },
        &mut violations,
    );

    violations
}

fn function_too_long(
    context: &LintContext<'_>,
    expr: &Expression,
    call: &Call,
) -> Option<RuleViolation> {
    let call_name = call.get_call_name(context);
    if matches!(call_name.as_str(), "def" | "export def")
        && let Some((function_name, _)) = call.extract_declaration_name(context)
    {
        let function_text = expr.span_text(context);

        // Check if function is on a single line and exceeds length limit
        if !function_text.contains('\n') && function_text.len() > MAX_FUNCTION_LINE_LENGTH {
            return Some(create_violation(&function_name, expr.span));
        }
    }
    None
}

fn create_violation(function_name: &str, span: nu_protocol::Span) -> RuleViolation {
    RuleViolation::new_dynamic(
        "prefer_multiline_functions",
        format!(
            "Function '{function_name}' is too long for a single line ({} characters)",
            span.end - span.start
        ),
        span,
    )
    .with_suggestion_static(
        "Break this function definition across multiple lines for better readability",
    )
}

/// This rule uses AST-based detection and is compatible with topiary-nushell
/// tree-sitter formatting. It provides more precise detection than regex-based
/// approaches and won't conflict with automatic formatters.
pub fn rule() -> Rule {
    Rule::new(
        "prefer_multiline_functions",
        LintLevel::Allow,
        "Prefer multiline format for long function definitions",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
