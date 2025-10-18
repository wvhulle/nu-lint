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
        let enabled_rules = self.get_enabled_rules();

        enabled_rules
            .flat_map(|rule| (rule.check)(context))
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
