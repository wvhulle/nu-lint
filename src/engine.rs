use std::{borrow::Cow, fs, path::Path, sync::OnceLock};

use nu_parser::parse;
use nu_protocol::{
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};

use crate::{
    LintError, LintLevel, config::Config, context::LintContext, rules::ALL_RULES,
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
    pub(crate) config: Config,
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
        log::debug!("Linting file: {}", path.display());
        let source = fs::read_to_string(path)?;
        let mut violations = self.lint_str(&source);

        let file_path: &str = path.to_str().unwrap();
        let file_path: Cow<'static, str> = file_path.to_owned().into();
        for violation in &mut violations {
            violation.file = Some(file_path.clone());
        }

        violations.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(a.lint_level.cmp(&b.lint_level))
        });
        Ok(violations)
    }

    #[must_use]
    pub fn lint_str(&self, source: &str) -> Vec<Violation> {
        let (block, working_set) = parse_source(self.engine_state, source.as_bytes());

        let context = LintContext {
            source,
            ast: &block,
            engine_state: self.engine_state,
            working_set: &working_set,
        };

        self.collect_violations(&context)
    }

    /// Collect violations from all enabled rules
    fn collect_violations(&self, context: &LintContext) -> Vec<Violation> {
        ALL_RULES
            .iter()
            .filter_map(|rule| {
                let lint_level = self.config.get_lint_level(rule.id);

                if lint_level == LintLevel::Allow {
                    return None;
                }

                let mut violations = (rule.check)(context);
                for violation in &mut violations {
                    violation.set_lint_level(lint_level);
                }

                (!violations.is_empty()).then_some(violations)
            })
            .flatten()
            .collect()
    }
}
