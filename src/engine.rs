use std::{path::Path, sync::OnceLock};

use nu_parser::parse;
use nu_protocol::{
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};

use crate::{
    LintError, config::Config, context::LintContext, lint::Violation, rules::RuleRegistry,
};

/// Parse Nushell source code into an AST and return both the Block and
/// `StateWorkingSet`.
fn parse_source<'a>(engine_state: &'a EngineState, source: &[u8]) -> (Block, StateWorkingSet<'a>) {
    let mut working_set = StateWorkingSet::new(engine_state);
    let block = parse(&mut working_set, None, source, false);

    ((*block).clone(), working_set)
}

pub struct LintEngine {
    registry: RuleRegistry,
    config: Config,
    engine_state: &'static EngineState,
}

impl LintEngine {
    /// Get or initialize the default engine state
    fn default_engine_state() -> &'static EngineState {
        static ENGINE: OnceLock<EngineState> = OnceLock::new();
        ENGINE.get_or_init(|| {
            let engine_state = nu_cmd_lang::create_default_context();
            nu_command::add_shell_command_context(engine_state)
        })
    }

    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            registry: RuleRegistry::with_default_rules(),
            config,
            engine_state: Self::default_engine_state(),
        }
    }

    /// Lint a file at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        let source = std::fs::read_to_string(path)?;
        Ok(self.lint_source(&source, Some(path)))
    }

    #[must_use]
    pub fn lint_source(&self, source: &str, path: Option<&Path>) -> Vec<Violation> {
        let (block, working_set) = parse_source(self.engine_state, source.as_bytes());

        let context = LintContext {
            source,
            file_path: path,
            ast: &block,
            engine_state: self.engine_state,
            working_set: &working_set,
        };

        let mut violations = self.collect_violations(&context);
        Self::attach_file_path(&mut violations, path);
        Self::sort_violations(&mut violations);
        violations
    }

    /// Collect violations from all enabled rules
    fn collect_violations(&self, context: &LintContext) -> Vec<Violation> {
        let eligible_rules = self.get_eligible_rules();

        eligible_rules
            .flat_map(|rule| {
                let rule_violations = (rule.check)(context);
                let rule_severity = self.get_effective_rule_severity(rule);

                // Convert RuleViolations to Violations with the rule's effective severity
                rule_violations
                    .into_iter()
                    .map(|rule_violation| rule_violation.into_violation(rule_severity))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Get all rules that are enabled according to the configuration
    fn get_enabled_rules(&self) -> impl Iterator<Item = &crate::rule::Rule> {
        self.registry.all_rules().filter(|rule| {
            // If not in config, use default (enabled). If in config, check if it's not
            // turned off.
            !matches!(
                self.config.rules.get(rule.id),
                Some(&crate::config::RuleSeverity::Off)
            )
        })
    }

    /// Get all rules that are enabled and meet the `min_severity` threshold
    /// This is more efficient as it avoids running rules that would be filtered
    /// out anyway
    fn get_eligible_rules(&self) -> impl Iterator<Item = &crate::rule::Rule> {
        let min_severity_threshold = self.get_minimum_severity_threshold();

        self.get_enabled_rules().filter(move |rule| {
            let rule_severity = self.get_effective_rule_severity(rule);

            // Handle special case: min_severity = "off" means no rules are eligible
            if matches!(
                self.config.general.min_severity,
                crate::config::RuleSeverity::Off
            ) {
                return false;
            }

            // Check if rule severity meets minimum threshold
            match min_severity_threshold {
                Some(min_threshold) => rule_severity >= min_threshold,
                None => true, // min_severity = "info" means all rules are eligible
            }
        })
    }

    /// Get the effective severity for a rule (config override or rule default)
    fn get_effective_rule_severity(&self, rule: &crate::rule::Rule) -> crate::lint::Severity {
        if let Some(config_severity) = self.config.rule_severity(rule.id) {
            config_severity
        } else {
            rule.severity
        }
    }

    /// Get the minimum severity threshold from `min_severity` config
    /// `min_severity` sets the minimum threshold for showing violations:
    /// - "error": Show only errors (minimum threshold = Error)
    /// - "warning": Show warnings and errors (minimum threshold = Warning)
    /// - "info": Show info, warnings, and errors (minimum threshold = Info,
    ///   i.e., all)
    /// - "off": Show nothing
    fn get_minimum_severity_threshold(&self) -> Option<crate::lint::Severity> {
        use crate::config::RuleSeverity;
        match self.config.general.min_severity {
            RuleSeverity::Error => Some(crate::lint::Severity::Error), // Show only errors
            RuleSeverity::Warning => Some(crate::lint::Severity::Warning), /* Show warnings and
                                                                             * above */
            RuleSeverity::Info => None, // Show all (no filtering)
            RuleSeverity::Off => None,  // Special case handled separately - show nothing
        }
    }

    /// Attach file path to all violations
    fn attach_file_path(violations: &mut [Violation], path: Option<&Path>) {
        if let Some(file_path_str) = path.and_then(|p| p.to_str()) {
            use std::borrow::Cow;
            let file_path: Cow<'static, str> = file_path_str.to_owned().into();
            for violation in violations {
                violation.file = Some(file_path.clone());
            }
        }
    }

    /// Sort violations by span start position, then by severity
    fn sort_violations(violations: &mut [Violation]) {
        violations.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(a.severity.cmp(&b.severity))
        });
    }

    #[must_use]
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_valid_code() {
        let engine = LintEngine::new(Config::default());
        let source = "let my_variable = 5";
        let violations = engine.lint_source(source, None);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_lint_invalid_snake_case() {
        let engine = LintEngine::new(Config::default());
        let source = "let myVariable = 5";
        let violations = engine.lint_source(source, None);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].rule_id, "snake_case_variables");
    }
}
