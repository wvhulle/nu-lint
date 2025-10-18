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
///
/// The `StateWorkingSet` contains the delta with newly defined declarations
/// (functions, aliases, etc.) which is essential for AST-based linting rules
/// that need to inspect function signatures, parameter types, and other
/// semantic information.
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

pub struct LintEngineBuilder {
    registry: Option<RuleRegistry>,
    config: Option<Config>,
    engine_state: Option<&'static EngineState>,
}

impl Default for LintEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LintEngineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: None,
            config: None,
            engine_state: None,
        }
    }

    #[must_use]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    #[must_use]
    pub fn with_registry(mut self, registry: RuleRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    #[must_use]
    pub fn with_engine_state(mut self, engine_state: &'static EngineState) -> Self {
        self.engine_state = Some(engine_state);
        self
    }

    #[must_use]
    pub fn engine_state() -> &'static EngineState {
        static ENGINE: OnceLock<EngineState> = OnceLock::new();
        ENGINE.get_or_init(|| {
            let engine_state = nu_cmd_lang::create_default_context();
            nu_command::add_shell_command_context(engine_state)
        })
    }

    #[must_use]
    pub fn build(self) -> LintEngine {
        LintEngine {
            registry: self
                .registry
                .unwrap_or_else(RuleRegistry::with_default_rules),
            config: self.config.unwrap_or_default(),
            engine_state: self.engine_state.unwrap_or_else(Self::engine_state),
        }
    }
}

impl LintEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        LintEngineBuilder::new().with_config(config).build()
    }

    #[must_use]
    pub fn builder() -> LintEngineBuilder {
        LintEngineBuilder::new()
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
            if let Some(configured_severity) = self.config.rule_severity(rule.id) {
                configured_severity == rule.severity
            } else {
                !self.config.rules.contains_key(rule.id)
            }
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
