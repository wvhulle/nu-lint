use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;
use std::sync::OnceLock;

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
mod tests {
    use super::*;

    #[test]
    fn test_pipe_spacing() {
        let rule = PipeSpacing::default();

        let good = "ls | get name | str upcase";
        let context = LintContext::test_from_source(good);
        assert_eq!(rule.check(&context).len(), 0);

        let bad = "ls|get name";
        let context = LintContext::test_from_source(bad);
        assert!(!rule.check(&context).is_empty());
    }

    #[test]
    fn test_closure_pipe_not_flagged() {
        let rule = PipeSpacing::default();

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
