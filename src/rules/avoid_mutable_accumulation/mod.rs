use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn mut_list_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"mut\s+(\w+)\s*=\s*\[\s*\]").unwrap())
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    mut_list_pattern()
        .captures_iter(context.source)
        .filter_map(|cap| {
            let var_name = cap.get(1)?.as_str();
            let append_pattern = format!(r"\${}.*\|\s*append", regex::escape(var_name));
            Regex::new(&append_pattern)
                .ok()?
                .is_match(context.source)
                .then(|| {
                    let full_match = cap.get(0)?;
                    Some(
                        RuleViolation::new_dynamic(
                            "avoid_mutable_accumulation",
                            format!(
                                "Mutable list '{var_name}' with append - consider using \
                                 functional pipeline"
                            ),
                            nu_protocol::Span::new(full_match.start(), full_match.end()),
                        )
                        .with_suggestion_static(
                            "Use '$items | each { ... }' instead of mutable accumulation",
                        ),
                    )
                })?
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "avoid_mutable_accumulation",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Prefer functional pipelines over mutable list accumulation",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
