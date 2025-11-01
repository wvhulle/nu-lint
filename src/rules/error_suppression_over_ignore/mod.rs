use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn file_operation_ignore_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        // Match file operations (rm, mv, cp, mkdir, rmdir, touch) followed by | ignore
        // This catches the common mistake where users think | ignore will suppress
        // errors
        Regex::new(r"\b(rm|mv|cp|mkdir|rmdir|touch)\b[^\|]*\|\s*ignore\s*(?:\n|$)").unwrap()
    })
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // Pattern: file_operation ... | ignore
    // People often use this thinking it will suppress errors, but it doesn't!
    // | ignore only discards stdout; errors still propagate and stop execution
    let pattern = file_operation_ignore_pattern();

    pattern
        .find_iter(context.source)
        .filter_map(|mat| {
            // Get context to check if it's an external command
            let context_start = mat.start().saturating_sub(10);
            let context_str = &context.source[context_start..mat.start()];

            // External commands have different error handling, so skip them
            let is_external = context_str.contains('^');

            if is_external {
                None
            } else {
                // Find the position of "| ignore" within the match
                let ignore_pos = mat.as_str().find("| ignore").unwrap_or(0);
                let ignore_start = mat.start() + ignore_pos;
                let ignore_end = ignore_start + "| ignore".len();

                // Extract the command portion (everything before | ignore)
                let command_text = mat.as_str()[..ignore_pos].trim();

                // Create a context-specific suggestion
                let suggestion = format!(
                    "'| ignore' only discards output, not errors. For error \
                     suppression:\n\nInstead of:  {command_text} | ignore\nUse:         do -i {{ \
                     {command_text} }}\n\nOr use try-catch for explicit error handling:\ntry {{ \
                     {command_text} }} catch {{ print 'failed' }}"
                );

                Some(
                    RuleViolation::new_dynamic(
                        "prefer_error_suppression_over_ignore",
                        "Using '| ignore' with file operations doesn't suppress errors - use 'do \
                         -i { ... }' instead"
                            .to_string(),
                        nu_protocol::Span::new(ignore_start, ignore_end),
                    )
                    .with_suggestion_dynamic(suggestion),
                )
            }
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_error_suppression_over_ignore",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "File operations with '| ignore' don't suppress errors - use 'do -i { ... }' instead",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
