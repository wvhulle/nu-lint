use regex::Regex;

use crate::{
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    // TODO: convert to AST
    let mut violations = Vec::new();

    // Pattern 1: split row followed by get/skip with index access
    let split_get_pattern =
        Regex::new(r#"split\s+row\s+["'][^"']*["']\s*\|\s*(get\s+\d+|skip\s+\d+)"#).unwrap();

    violations.extend(context.violations_from_regex(
        &split_get_pattern,
        "prefer_parse_command",
        |_| {
            Some((
                "Manual string splitting with indexed access - consider using 'parse'".to_string(),
                Some(
                    "Use 'parse \"pattern {field1} {field2}\"' for structured text extraction"
                        .to_string(),
                ),
            ))
        },
    ));

    // Pattern 2: let parts = ... split row, then $parts | get
    let split_to_var_pattern = Regex::new(r"let\s+(\w+)\s*=\s*\([^)]*split\s+row[^)]*\)").unwrap();

    violations.extend(
        split_to_var_pattern
            .find_iter(context.source)
            .filter_map(|mat| {
                let var_name = mat.as_str().split_whitespace().nth(1)?;
                // Look for subsequent indexed access
                let access_pattern =
                    format!(r"\${}?\s*\|\s*(get|skip)\s+\d+", regex::escape(var_name));

                if Regex::new(&access_pattern).ok()?.is_match(context.source) {
                    Some(
                        RuleViolation::new_dynamic(
                            "prefer_parse_command",
                            format!(
                                "Variable '{var_name}' from split row with indexed access - \
                                 consider using 'parse'"
                            ),
                            nu_protocol::Span::new(mat.start(), mat.end()),
                        )
                        .with_suggestion_static(
                            "Use 'parse' command to extract named fields instead of indexed access",
                        ),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    );

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_parse_command",
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer 'parse' command over manual string splitting with indexed access",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
