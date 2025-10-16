use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct AvoidMutableAccumulation;

impl AvoidMutableAccumulation {
    fn mut_list_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"mut\s+(\w+)\s*=\s*\[\s*\]").unwrap())
    }
}

impl Rule for AvoidMutableAccumulation {
    fn id(&self) -> &'static str {
        "BP002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Prefer functional pipelines over mutable list accumulation"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        Self::mut_list_pattern()
            .captures_iter(context.source)
            .filter_map(|cap| {
                let var_name = cap.get(1)?.as_str();
                let append_pattern = format!(r"\${}.*\|\s*append", regex::escape(var_name));
                Regex::new(&append_pattern).ok()?.is_match(context.source).then(|| {
                    let full_match = cap.get(0)?;
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!("Mutable list '{var_name}' with append - consider using functional pipeline"),
                        span: nu_protocol::Span::new(full_match.start(), full_match.end()),
                        suggestion: Some("Use '$items | each { ... }' instead of mutable accumulation".to_string()),
                        fix: None,
                        file: None,
                    })
                })?
            })
            .collect()
    }
}
