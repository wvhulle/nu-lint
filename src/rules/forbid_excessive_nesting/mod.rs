use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

const MAX_INDENTATION_LEVELS: usize = 4;

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let source = context.source;
    let mut violations = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let indentation_level = count_indentation_level(line);

        if indentation_level > MAX_INDENTATION_LEVELS {
            let line_start = source
                .lines()
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();

            let span = nu_protocol::Span::new(line_start, line_start + line.len());

            violations.push(
                RuleViolation::new_dynamic(
                    "forbid_excessive_nesting",
                    format!(
                        "Line has {indentation_level} levels of indentation, which exceeds the \
                         maximum of {MAX_INDENTATION_LEVELS}"
                    ),
                    span,
                )
                .with_suggestion_static(
                    "Consider refactoring this code into smaller functions to reduce nesting depth",
                ),
            );
        }
    }

    violations
}

fn count_indentation_level(line: &str) -> usize {
    let leading_spaces = line.len() - line.trim_start().len();
    leading_spaces / 2
}

pub fn rule() -> Rule {
    Rule::new(
        "forbid_excessive_nesting",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Avoid excessive nesting (more than 4 indentation levels)",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
