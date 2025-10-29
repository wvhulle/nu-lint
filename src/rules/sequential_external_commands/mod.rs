use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Find all external command calls in the AST
fn find_external_commands(context: &LintContext) -> Vec<Span> {
    use nu_protocol::ast::Traverse;

    let mut sequential_externals = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::ExternalCall(_head, _args) = &expr.expr {
                return vec![expr.span];
            }
            vec![]
        },
        &mut sequential_externals,
    );

    sequential_externals
}

/// Check if text contains error handling patterns
fn has_error_handling_between(text: &str) -> bool {
    text.contains("complete")
        || text.contains("try")
        || text.contains("&&")
        || text.contains("exit_code")
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    const MAX_DISTANCE: usize = 200;

    let sequential_externals = find_external_commands(context);
    let mut violations = Vec::new();
    let mut seen_pairs = std::collections::HashSet::new();

    for i in 0..sequential_externals.len() {
        for j in (i + 1)..sequential_externals.len() {
            let first_span = sequential_externals[i];
            let second_span = sequential_externals[j];

            // Skip if we've already checked this pair
            let pair_key = (first_span.start, second_span.start);
            if seen_pairs.contains(&pair_key) {
                continue;
            }

            if second_span.start - first_span.end < MAX_DISTANCE {
                let between_text = &context.source[first_span.end..second_span.start];
                if !has_error_handling_between(between_text) {
                    let combined_span = nu_protocol::Span::new(first_span.start, second_span.end);
                    violations.push(
                        RuleViolation::new_static(
                            "sequential_external_commands",
                            "Sequential external commands without error checking - failures in \
                             first command ignored",
                            combined_span,
                        )
                        .with_suggestion_static(
                            "Check exit codes between commands or use '&&' for conditional \
                             execution",
                        ),
                    );
                    seen_pairs.insert(pair_key);
                    break; // Only report one violation per first command
                }
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "sequential_external_commands",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Ensure sequential external commands have error checking between them",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
