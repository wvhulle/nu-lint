use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;
use std::sync::OnceLock;

pub struct PipeSpacing;

impl PipeSpacing {
    pub fn new() -> Self {
        Self
    }

    fn bad_pipe_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"(\S\||  +\||\|  +|\|\S)").unwrap())
    }

    fn closure_param_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\{\s*\|[^|]*\|").unwrap())
    }
}

impl Default for PipeSpacing {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PipeSpacing {
    fn id(&self) -> &str {
        "S005"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Pipes should have exactly one space before and after"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let bad_pipe_pattern = Self::bad_pipe_pattern();
        let closure_param_pattern = Self::closure_param_pattern();
        let closure_regions: Vec<(usize, usize)> = closure_param_pattern
            .find_iter(context.source)
            .map(|m| (m.start(), m.end()))
            .collect();

        context.violations_from_regex_if(bad_pipe_pattern, self.id(), self.severity(), |mat| {
            // Skip if this pipe is inside a closure parameter list
            let in_closure = closure_regions
                .iter()
                .any(|(c_start, c_end)| mat.start() >= *c_start && mat.end() <= *c_end);

            if in_closure {
                return None;
            }

            let text = mat.as_str();
            let message = if text.starts_with('|')
                && text.len() > 1
                && !text.chars().nth(1).unwrap().is_whitespace()
            {
                "Pipe should have space after |"
            } else if text.ends_with('|')
                && text.len() > 1
                && !text.chars().nth(text.len() - 2).unwrap().is_whitespace()
            {
                "Pipe should have space before |"
            } else {
                "Pipe should have exactly one space before and after"
            };

            Some((
                message.to_string(),
                Some("Use ' | ' with single spaces".to_string()),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_spacing() {
        let rule = PipeSpacing::new();

        let good = "ls | get name | str upcase";
        let context = LintContext::test_from_source(good);
        assert_eq!(rule.check(&context).len(), 0);

        let bad = "ls|get name";
        let context = LintContext::test_from_source(bad);
        assert!(!rule.check(&context).is_empty());
    }

    #[test]
    fn test_closure_pipe_not_flagged() {
        let rule = PipeSpacing::new();

        // Closures with parameters should not be flagged
        let closure_code = "let completer = {|spans| echo $spans}";
        let context = LintContext::test_from_source(closure_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Closure parameter pipes should not be flagged"
        );

        // Multiple parameter closures
        let multi_param = "{|x, y| $x + $y}";
        let context = LintContext::test_from_source(multi_param);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Multi-param closure pipes should not be flagged"
        );

        // But actual pipe operators should still be flagged
        let bad_with_closure = "{|x| echo $x}|get name";
        let context = LintContext::test_from_source(bad_with_closure);
        assert!(
            !rule.check(&context).is_empty(),
            "Pipe operators should still be flagged"
        );
    }
}
