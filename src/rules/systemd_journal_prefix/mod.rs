use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn print_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(?:^|[;\n])\s*print\s+(?:"([^"]*)"|'([^']*)'|(\S+))"#).unwrap()
    })
}

fn echo_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(?:^|[;\n])\s*echo\s+(?:"([^"]*)"|'([^']*)'|(\S+))"#).unwrap()
    })
}

fn has_journal_prefix(text: &str) -> bool {
    static PREFIX_PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PREFIX_PATTERN.get_or_init(|| {
        // Match either numeric prefix <0-7> or keyword prefix (emerg, alert, crit, err, warning, notice, info, debug)
        Regex::new(r"^<(?:[0-7]|emerg|alert|crit|err|warning|notice|info|debug)>").unwrap()
    });
    pattern.is_match(text)
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();
    let print_pat = print_pattern();
    let echo_pat = echo_pattern();

    for mat in print_pat.find_iter(context.source) {
        let full_match = mat.as_str();

        if let Some(caps) = print_pat.captures(full_match) {
            let output_text = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map_or("", |m| m.as_str());

            if !has_journal_prefix(output_text) {
                violations.push(
                    RuleViolation::new_static(
                        "systemd_journal_prefix",
                        "Output without systemd journal log level prefix - consider adding prefix for proper logging",
                        nu_protocol::Span::new(mat.start(), mat.end()),
                    )
                    .with_suggestion_static(
                        "Add systemd journal prefix using numbers <0-7> or keywords: <emerg>, <alert>, <crit>, <err>, <warning>, <notice>, <info>, <debug>. Example: print \"<info>Starting process\"",
                    ),
                );
            }
        }
    }

    for mat in echo_pat.find_iter(context.source) {
        let full_match = mat.as_str();

        if let Some(caps) = echo_pat.captures(full_match) {
            let output_text = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map_or("", |m| m.as_str());

            if !has_journal_prefix(output_text) {
                violations.push(
                    RuleViolation::new_static(
                        "systemd_journal_prefix",
                        "Output without systemd journal log level prefix - consider adding prefix for proper logging",
                        nu_protocol::Span::new(mat.start(), mat.end()),
                    )
                    .with_suggestion_static(
                        "Add systemd journal prefix using numbers <0-7> or keywords: <emerg>, <alert>, <crit>, <err>, <warning>, <notice>, <info>, <debug>. Example: echo \"<info>Starting process\"",
                    ),
                );
            }
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "systemd_journal_prefix",
        RuleCategory::Idioms,
        Severity::Warning,
        "Detect output without systemd journal log level prefix when using SyslogLevelPrefix",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
