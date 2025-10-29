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

fn create_violation_if_needed(
    first_span: Span,
    second_span: Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    let between_text = &context.source[first_span.end..second_span.start];
    if has_error_handling_between(between_text) {
        return None;
    }

    let combined_span = nu_protocol::Span::new(first_span.start, second_span.end);
    Some(
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
    )
}

fn process_external_command_pair(
    i: usize,
    sequential_externals: &[Span],
    seen_pairs: &mut std::collections::HashSet<(usize, usize)>,
    context: &LintContext,
) -> Option<RuleViolation> {
    const MAX_DISTANCE: usize = 200;

    for j in (i + 1)..sequential_externals.len() {
        let first_span = sequential_externals[i];
        let second_span = sequential_externals[j];
        let pair_key = (first_span.start, second_span.start);

        if seen_pairs.contains(&pair_key) || second_span.start - first_span.end >= MAX_DISTANCE {
            continue;
        }

        if let Some(violation) = create_violation_if_needed(first_span, second_span, context) {
            seen_pairs.insert(pair_key);
            return Some(violation);
        }
    }
    None
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let sequential_externals = find_external_commands(context);
    let mut seen_pairs = std::collections::HashSet::new();

    (0..sequential_externals.len())
        .filter_map(|i| {
            process_external_command_pair(i, &sequential_externals, &mut seen_pairs, context)
        })
        .collect()
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
