use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check_brace_spacing(source: &str, span: Span, has_params: bool) -> Vec<Violation> {
    let mut violations = Vec::new();

    if span.start >= span.end || span.end > source.len() {
        return violations;
    }

    let text = &source[span.start..span.end];

    // Find opening and closing braces
    if !text.starts_with('{') || !text.ends_with('}') {
        return violations;
    }

    let inner = &text[1..text.len() - 1];

    // Empty braces are fine
    if inner.trim().is_empty() {
        return violations;
    }

    // Check for space after opening brace before closure parameters
    if has_params && inner.starts_with(|c: char| c.is_whitespace()) {
        let pipe_pos = inner.find('|');
        if let Some(pos) = pipe_pos
            && pos > 0
            && inner[..pos].trim().is_empty()
        {
            violations.push(Violation {
                rule_id: "brace_spacing".into(),
                severity: Severity::Info,
                message: "No space allowed after opening brace before closure parameters"
                    .to_string()
                    .into(),
                span,
                suggestion: Some("Use {|param| instead of { |param|".to_string().into()),
                fix: None,
                file: None,
            });
            return violations;
        }
    }

    // Skip closure parameter checking for other cases
    if has_params {
        return violations;
    }

    // Check for inconsistent spacing in records/blocks
    let starts_with_space = inner.starts_with(|c: char| c.is_whitespace());
    let ends_with_space = inner.ends_with(|c: char| c.is_whitespace());

    // Inconsistent: one has space, the other doesn't
    if starts_with_space != ends_with_space {
        violations.push(Violation {
            rule_id: "brace_spacing".into(),
            severity: Severity::Info,
            message: "Inconsistent brace spacing: use either {x} or { x }, not { x} or {x }"
                .to_string()
                .into(),
            span,
            suggestion: Some(
                "Use consistent spacing: both spaces or no spaces inside braces"
                    .to_string()
                    .into(),
            ),
            fix: None,
            file: None,
        });
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| {
        match &expr.expr {
            // Closures and blocks with parameters
            Expr::Closure(block_id) | Expr::Block(block_id) => {
                let block = ctx.working_set.get_block(*block_id);
                let has_params = !block.signature.required_positional.is_empty()
                    || !block.signature.optional_positional.is_empty()
                    || block.signature.rest_positional.is_some();

                check_brace_spacing(ctx.source, expr.span, has_params)
            }
            // Records
            Expr::Record(items) => {
                if items.is_empty() {
                    vec![]
                } else {
                    check_brace_spacing(ctx.source, expr.span, false)
                }
            }
            _ => vec![],
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "brace_spacing",
        RuleCategory::Formatting,
        Severity::Info,
        "Braces should have consistent spacing: either {x} or { x }, and no space before closure \
         parameters",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
