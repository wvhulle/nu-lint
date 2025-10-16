use crate::config::Config;
use crate::context::{LintContext, LintError, Violation};
use crate::parser::parse_source;
use crate::rules::RuleRegistry;
use nu_protocol::engine::EngineState;
use std::path::Path;
use std::sync::OnceLock;

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

    pub fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        let source = std::fs::read_to_string(path)?;
        Ok(self.lint_source(&source, Some(path)))
    }

    #[must_use]
    pub fn lint_source(&self, source: &str, path: Option<&Path>) -> Vec<Violation> {
        let (block, working_set) = parse_source(self.engine_state, source.as_bytes());

        let context = LintContext {
            source,
            ast: &block,
            engine_state: self.engine_state,
            working_set: &working_set,
            file_path: path,
        };

        let mut violations = Vec::new();
        let file_path = path.and_then(|p| p.to_str()).map(String::from);

        for rule in self.registry.all_rules() {
            if let Some(configured_severity) = self.config.rule_severity(rule.id()) {
                if configured_severity != rule.severity() {
                    continue;
                }
            } else if self.config.rules.contains_key(rule.id()) {
                continue;
            }

            let mut rule_violations = rule.check(&context);

            for violation in &mut rule_violations {
                violation.file.clone_from(&file_path);
            }

            violations.extend(rule_violations);
        }

        violations.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(a.severity.cmp(&b.severity))
        });

        violations
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
        assert_eq!(violations[0].rule_id, "S001");
    }
}
