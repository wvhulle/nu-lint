use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Pattern: if $var == value { } else if $var == value { } else { }
    // Since backreferences aren't supported, capture both variables and check
    // manually
    let if_chain_pattern =
        Regex::new(r"if\s+\$(\w+)\s*==\s*[^\{]+\{[^\}]*\}\s*else\s+if\s+\$(\w+)\s*==").unwrap();

    violations.extend(context.violations_from_regex_if(
        &if_chain_pattern,
        "prefer_match_over_if_chain",
        Severity::Info,
        |mat| {
            let caps = if_chain_pattern.captures(mat.as_str())?;
            let var_name1 = &caps[1];
            let var_name2 = &caps[2];

            // Check if it's the same variable being compared
            if var_name1 == var_name2 {
                Some((
                    format!(
                        "If-else-if chain comparing '{var_name1}' to different values - \
                         consider using 'match'"
                    ),
                    Some(
                        "Use 'match $var { value1 => { ... }, value2 => { ... }, _ => { ... } \
                         }' for clearer value-based branching"
                            .to_string(),
                    ),
                ))
            } else {
                None
            }
        },
    ));

    // Also detect multiple else-if chains (3+ branches) even if variable changes
    let multiple_else_if =
        Regex::new(r"if\s+[^\{]+\{[^\}]*\}\s*else\s+if\s+[^\{]+\{[^\}]*\}\s*else\s+if")
            .unwrap();

    violations.extend(context.violations_from_regex_if(
        &multiple_else_if,
        "prefer_match_over_if_chain",
        Severity::Info,
        |mat| {
            // Check if we already reported this location
            let already_reported = violations
                .iter()
                .any(|v| v.span.start <= mat.start() && mat.start() <= v.span.end);

            if already_reported {
                None
            } else {
                Some((
                    "Long if-else-if chain - consider using 'match' for clearer branching"
                        .to_string(),
                    Some(
                        "For multiple related conditions, 'match' provides clearer pattern \
                         matching"
                            .to_string(),
                    ),
                ))
            }
        },
    ));

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_match_over_if_chain",
        RuleCategory::Idioms,
        Severity::Info,
        "Use 'match' for value-based branching instead of if-else-if chains",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
