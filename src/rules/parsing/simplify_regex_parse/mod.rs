use std::sync::LazyLock;

use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr},
};
use regex::Regex;

use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, regex::contains_regex_special_chars, string::StringFormat},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

pub struct FixData {
    span: Span,
    simplified_pattern: String,
}

fn extract_regex_pattern(call: &Call, context: &LintContext) -> Option<String> {
    if !call.is_call_to_command("parse", context) {
        return None;
    }

    let has_regex_flag = call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Named((name, _, _)) if name.item == "regex"));

    if !has_regex_flag {
        return None;
    }

    let pattern_arg = call.get_first_positional_arg()?;

    match &pattern_arg.expr {
        Expr::String(s) | Expr::RawString(s) => Some(s.clone()),
        _ => {
            StringFormat::from_expression(pattern_arg, context).map(|fmt| fmt.content().to_string())
        }
    }
}

fn can_simplify_regex_pattern(pattern: &str) -> Option<String> {
    static CAPTURE_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\(\?P<([^>]+)>\.\*\)").unwrap());

    Regex::new(pattern).ok()?;

    let captures: Vec<_> = CAPTURE_PATTERN.captures_iter(pattern).collect();
    if captures.is_empty() {
        return None;
    }

    let mut simplified = String::new();
    let mut last_end = 0;

    for cap in captures {
        let full_match = cap.get(0)?;
        let name = cap.get(1)?.as_str();

        let delimiter = &pattern[last_end..full_match.start()];
        if contains_regex_special_chars(delimiter) {
            return None;
        }

        simplified.push_str(delimiter);
        simplified.push('{');
        simplified.push_str(name);
        simplified.push('}');

        last_end = full_match.end();
    }

    let trailing = &pattern[last_end..];
    if !trailing.is_empty() && contains_regex_special_chars(trailing) {
        return None;
    }
    simplified.push_str(trailing);

    Some(simplified)
}

fn check_parse_regex(expr: &Expr, ctx: &LintContext) -> Option<(Detection, FixData)> {
    let Expr::Call(call) = expr else {
        return None;
    };

    let pattern = extract_regex_pattern(call, ctx)?;
    let simplified = can_simplify_regex_pattern(&pattern)?;

    let violation = Detection::from_global_span(
        "Simplify 'parse --regex' to 'parse' with pattern syntax",
        call.span(),
    )
    .with_primary_label("regex not needed for simple delimiters");

    Some((
        violation,
        FixData {
            span: call.span(),
            simplified_pattern: simplified,
        },
    ))
}

struct SimplifyRegexRule;

impl DetectFix for SimplifyRegexRule {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "simplify_regex_parse"
    }

    fn short_description(&self) -> &'static str {
        "Simplify 'parse --regex' to 'parse' with pattern syntax"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "When a 'parse --regex' pattern only uses simple capture groups like '(?P<name>.*)' \
             separated by literal delimiters, it can be rewritten using the simpler '{name}' \
             pattern syntax without the --regex flag.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            check_parse_regex(&expr.expr, ctx).into_iter().collect()
        })
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = format!("parse \"{}\"", fix_data.simplified_pattern);

        Some(Fix {
            explanation: format!(
                "Simplify 'parse --regex' to 'parse \"{}\"'",
                fix_data.simplified_pattern
            )
            .into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &SimplifyRegexRule;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
