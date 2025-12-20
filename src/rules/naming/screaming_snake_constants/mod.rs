use std::sync::OnceLock;

use heck::ToShoutySnakeCase;
use regex::Regex;

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn screaming_snake_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap())
}
fn const_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\bconst\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap())
}
fn is_valid_screaming_snake(name: &str) -> bool {
    screaming_snake_pattern().is_match(name)
}
fn check(context: &LintContext) -> Vec<Violation> {
    const_pattern()
        .captures_iter(context.whole_source())
        .filter_map(|cap| {
            let const_match = cap.get(1)?;
            let const_name = const_match.as_str();
            if is_valid_screaming_snake(const_name) {
                None
            } else {
                Some(
                    Violation::new(
                        format!(
                            "Constant '{const_name}' should use SCREAMING_SNAKE_CASE naming \
                             convention"
                        ),
                        nu_protocol::Span::new(const_match.start(), const_match.end()),
                    )
                    .with_primary_label("non-SCREAMING_SNAKE_CASE")
                    .with_help(format!(
                        "Consider renaming to: {}",
                        const_name.to_shouty_snake_case()
                    )),
                )
            }
        })
        .collect()
}
pub const fn rule() -> Rule {
    Rule::new(
        "screaming_snake_constants",
        "Constants should use SCREAMING_SNAKE_CASE naming convention",
        check,
    )
    .with_doc_url("https://www.nushell.sh/book/style_guide.html#environment-variables")
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
