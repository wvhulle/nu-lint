use std::{collections::HashSet, fs, path::Path, sync::OnceLock};

use nu_parser::parse;
use nu_protocol::{
    ParseError,
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};

use crate::{
    LintError, RuleViolation,
    config::{Config, LintLevel},
    context::LintContext,
    rules::RuleRegistry,
    violation::Violation,
};

/// Parse Nushell source code into an AST and return both the Block and
/// `StateWorkingSet`.
fn parse_source<'a>(engine_state: &'a EngineState, source: &[u8]) -> (Block, StateWorkingSet<'a>) {
    let mut working_set = StateWorkingSet::new(engine_state);
    let block = parse(&mut working_set, None, source, false);

    ((*block).clone(), working_set)
}

pub struct LintEngine {
    pub registry: RuleRegistry,
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
    pub(crate) fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        let source = fs::read_to_string(path)?;
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
        self.registry
            .all_rules()
            .filter_map(|rule| {
                // Get the effective lint level for this rule
                let lint_level = self.config.get_lint_level(rule.id, rule.default_lint_level);

                // Run the rule and convert violations
                let rule_violations = (rule.check)(context);
                let violations: Vec<_> = rule_violations
                    .into_iter()
                    .map(|rule_violation| rule_violation.into_violation(lint_level))
                    .collect();

                (!violations.is_empty()).then_some(violations)
            })
            .flatten()
            .collect()
    }

    /// Convert parse errors from the `StateWorkingSet` into violations
    fn convert_parse_errors_to_violations(&self, working_set: &StateWorkingSet) -> Vec<Violation> {
        // Get the nu_parse_error rule to use its metadata
        let Some(rule) = self.registry.get_rule("nu_parse_error") else {
            return vec![];
        };

        // Get the effective lint level for nu_parse_error
        let lint_level = self.config.get_lint_level(rule.id, rule.default_lint_level);

        // If the rule is allowed, don't report parse errors
        if let LintLevel::Allow = lint_level {
            return vec![];
        }

        let mut seen = HashSet::new();

        // Convert each parse error to a violation, deduplicating by span and message
        // Filter out module-related errors since the linter works at AST level only
        working_set
            .parse_errors
            .iter()
            .filter(|parse_error| !matches!(parse_error, ParseError::ModuleNotFound(_, _)))
            .filter_map(|parse_error| {
                let key = (
                    parse_error.span().start,
                    parse_error.span().end,
                    parse_error.to_string(),
                );
                seen.insert(key).then(|| {
                    RuleViolation::new_dynamic(
                        "nu_parse_error",
                        parse_error.to_string(),
                        parse_error.span(),
                    )
                    .into_violation(lint_level)
                })
            })
            .collect()
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
                .then(a.lint_level.cmp(&b.lint_level))
        });
    }
}
