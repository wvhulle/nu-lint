use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

#[derive(Default)]
pub struct PipeSpacing;

impl PipeSpacing {
    fn bad_pipe_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"(\S\||  +\||\|  +|\|\S)").unwrap())
    }

    fn closure_param_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\{\s*\|[^|]*\|").unwrap())
    }
}

impl Rule for PipeSpacing {
    fn id(&self) -> &'static str {
        "pipe_spacing"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Pipes should have exactly one space before and after"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let closure_regions: Vec<_> = Self::closure_param_pattern()
            .find_iter(context.source)
            .map(|m| (m.start(), m.end()))
            .collect();

        context.violations_from_regex_if(
            Self::bad_pipe_pattern(),
            self.id(),
            self.severity(),
            |mat| {
                // Skip if this pipe is inside a closure parameter list
                if closure_regions
                    .iter()
                    .any(|&(start, end)| mat.start() >= start && mat.end() <= end)
                {
                    return None;
                }

                let text = mat.as_str();
                let message = match text.chars().next() {
                    Some('|') if text.len() > 1 && !text.chars().nth(1)?.is_whitespace() => {
                        "Pipe should have space after |"
                    }
                    _ if text.ends_with('|')
                        && text.len() > 1
                        && !text.chars().nth(text.len() - 2)?.is_whitespace() =>
                    {
                        "Pipe should have space before |"
                    }
                    _ => "Pipe should have exactly one space before and after",
                };

                Some((
                    message.to_string(),
                    Some("Use ' | ' with single spaces".to_string()),
                ))
            },
        )
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
