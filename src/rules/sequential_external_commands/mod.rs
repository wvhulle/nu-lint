use nu_protocol::{Span, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Find all external command calls in the AST along with their parent expression spans
fn find_external_commands(context: &LintContext) -> Vec<(Span, Span)> {
    use nu_protocol::ast::Traverse;

    let mut sequential_externals = Vec::new();
    context.ast.flat_map(
        context.working_set,
        &|expr| {
            if let Expr::ExternalCall(_head, _args) = &expr.expr {
                // Return both the external call span and the parent expression span
                // This helps us check if it's wrapped in complete/try
                return vec![(expr.span, expr.span)];
            }
            vec![]
        },
        &mut sequential_externals,
    );

    sequential_externals
}

/// Check if an external command is wrapped in error handling
fn is_wrapped_in_error_handling(span: Span, context: &LintContext) -> bool {
    // Get a safe substring by only looking at full source sections
    let source_before = &context.source[..span.start];
    let source_after = &context.source[span.end..];
    
    // Look at the last 100 chars before (safe because we're taking from the start)
    let prefix_text = if source_before.len() > 100 {
        &source_before[source_before.len() - 100..]
    } else {
        source_before
    };
    
    // Look at the first 100 chars after (safe because we're taking from the end)
    let suffix_text = if source_after.len() > 100 {
        &source_after[..100]
    } else {
        source_after
    };
    
    // Check for various error handling patterns
    (prefix_text.contains("try {") || prefix_text.contains("try{"))
        || suffix_text.trim_start().starts_with("| complete")
        || suffix_text.contains("| complete)")
}

/// Check if text contains error handling patterns between commands
fn has_error_handling_between(text: &str) -> bool {
    text.contains("exit_code")
        || text.contains("LAST_EXIT_CODE")
        || text.contains("try")
        || text.contains("&&")
        || text.contains("complete")
}

fn create_violation_if_needed(
    first_span: Span,
    second_span: Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    // First check if either command is wrapped in error handling
    if is_wrapped_in_error_handling(first_span, context)
        || is_wrapped_in_error_handling(second_span, context)
    {
        return None;
    }

    // Then check for error handling between the commands
    let between_text = &context.source[first_span.end..second_span.start];
    if has_error_handling_between(between_text) {
        return None;
    }

    // Use the first command's span for the violation location
    Some(
        RuleViolation::new_static(
            "sequential_external_commands",
            "Sequential external commands without error checking - failures in first command \
             ignored",
            first_span,
        )
        .with_suggestion_static(
            "Check exit codes using 'try', 'complete', or check $env.LAST_EXIT_CODE between \
             commands",
        ),
    )
}

fn process_external_command_pair(
    i: usize,
    sequential_externals: &[(Span, Span)],
    seen_pairs: &mut std::collections::HashSet<(usize, usize)>,
    context: &LintContext,
) -> Option<RuleViolation> {
    const MAX_DISTANCE: usize = 200;

    // Only check the immediate next external command to avoid duplicate violations
    if i + 1 >= sequential_externals.len() {
        return None;
    }

    let (first_span, _) = sequential_externals[i];
    let (second_span, _) = sequential_externals[i + 1];
    let pair_key = (first_span.start, second_span.start);

    if seen_pairs.contains(&pair_key)
        || second_span.start <= first_span.end
        || second_span.start - first_span.end >= MAX_DISTANCE
    {
        return None;
    }

    if let Some(violation) = create_violation_if_needed(first_span, second_span, context) {
        seen_pairs.insert(pair_key);
        return Some(violation);
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
