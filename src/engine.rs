use crate::config::Config;
use crate::context::{LintContext, LintError, Violation};
use crate::parser::parse_source;
use crate::rules::RuleRegistry;
use nu_protocol::engine::EngineState;
use std::path::Path;
use std::sync::OnceLock;

/// Create an engine state with standard library commands
/// This is cached since creating it is expensive
fn create_engine_state_with_stdlib() -> EngineState {
    // Create engine state with core language commands and full standard library
    // This follows the pattern used in nushell's test_bins.rs
    let engine_state = nu_cmd_lang::create_default_context();
    nu_command::add_shell_command_context(engine_state)
}

/// Get a cached engine state with standard library
fn get_stdlib_engine() -> &'static EngineState {
    static ENGINE: OnceLock<EngineState> = OnceLock::new();
    ENGINE.get_or_init(create_engine_state_with_stdlib)
}

pub struct LintEngine {
    registry: RuleRegistry,
    config: Config,
    engine_state: &'static EngineState,
}

impl LintEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            registry: RuleRegistry::with_default_rules(),
            config,
            engine_state: get_stdlib_engine(),
        }
    }

    pub fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        let source = std::fs::read_to_string(path)?;
        self.lint_source(&source, Some(path))
    }

    pub fn lint_source(
        &self,
        source: &str,
        path: Option<&Path>,
    ) -> Result<Vec<Violation>, LintError> {
        let (block, working_set) =
            parse_source(self.engine_state, source.as_bytes()).map_err(LintError::ParseError)?;

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

        Ok(violations)
    }

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
        let result = engine.lint_source(source, None);
        assert!(result.is_ok());
        let violations = result.unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_lint_invalid_snake_case() {
        let engine = LintEngine::new(Config::default());
        let source = "let myVariable = 5";
        let result = engine.lint_source(source, None);
        assert!(result.is_ok());
        let violations = result.unwrap();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].rule_id, "S001");
    }
}
