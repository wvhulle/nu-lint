use std::{collections::HashSet, path::Path, sync::OnceLock};

use nu_parser::parse;
use nu_protocol::{
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};

use crate::{
    LintError, config::Config, context::LintContext, rules::RuleRegistry, violation::Violation,
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
            let engine_state = nu_command::add_shell_command_context(engine_state);
            let mut engine_state = nu_cli::add_cli_context(engine_state);

            // Add print command (it's in nu-cli but not added by add_cli_context)
            let delta = {
                let mut working_set = StateWorkingSet::new(&engine_state);
                working_set.add_decl(Box::new(nu_cli::Print));
                working_set.render()
            };

            if let Err(err) = engine_state.merge_delta(delta) {
                eprintln!("Error adding Print command: {err:?}");
            }

            engine_state
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

        // Extract parse errors from the working set and convert to violations
        violations.extend(self.convert_parse_errors_to_violations(&working_set));

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

    /// Convert parse errors from the `StateWorkingSet` into violations
    fn convert_parse_errors_to_violations(&self, working_set: &StateWorkingSet) -> Vec<Violation> {
        // Get the nu_parse_error rule to use its metadata
        let parse_error_rule = self.registry.get_rule("nu_parse_error");

        if parse_error_rule.is_none() {
            return vec![];
        }

        let rule = parse_error_rule.unwrap();
        let rule_severity = self.get_effective_rule_severity(rule);

        // Check if this rule meets the minimum severity threshold
        if let Some(min_threshold) = self.get_minimum_severity_threshold()
            && rule_severity < min_threshold
        {
            return vec![];
        }

        let mut seen = HashSet::new();

        // Convert each parse error to a violation, deduplicating by span and message
        working_set
            .parse_errors
            .iter()
            .filter_map(|parse_error| {
                let key = (
                    parse_error.span().start,
                    parse_error.span().end,
                    parse_error.to_string(),
                );
                if seen.insert(key.clone()) {
                    use crate::violation::RuleViolation;

                    Some(
                        RuleViolation::new_dynamic(
                            "nu_parse_error",
                            parse_error.to_string(),
                            parse_error.span(),
                        )
                        .into_violation(rule_severity),
                    )
                } else {
                    None
                }
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
    fn get_effective_rule_severity(&self, rule: &crate::rule::Rule) -> crate::violation::Severity {
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
    fn get_minimum_severity_threshold(&self) -> Option<crate::violation::Severity> {
        use crate::config::RuleSeverity;
        match self.config.general.min_severity {
            RuleSeverity::Error => Some(crate::violation::Severity::Error), // Show only errors
            RuleSeverity::Warning => Some(crate::violation::Severity::Warning), // Show warnings and
            // above
            RuleSeverity::Info | RuleSeverity::Off => None, // Show all (no filtering)
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
