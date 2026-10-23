use std::{
    env,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use nu_parser::parse;
use nu_protocol::{
    Span, Value,
    ast::Block,
    engine::{EngineState, FileStack, StateWorkingSet},
};
use rayon::prelude::*;

use crate::{
    LintLevel,
    config::Config,
    context::LintContext,
    ignore,
    rule::{Rule, RuleCatalog},
    violation::{SourceFile, Violation},
};

#[derive(Debug)]
pub struct ReadSourceError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for ReadSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to read '{}': {}",
            self.path.display(),
            self.source
        )
    }
}

impl Error for ReadSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[must_use]
pub fn default_engine_state() -> &'static EngineState {
    static ENGINE: LazyLock<EngineState> = LazyLock::new(|| {
        let mut engine_state = nu_cmd_lang::create_default_context();
        engine_state = nu_command::add_shell_command_context(engine_state);
        engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
        engine_state = nu_cli::add_cli_context(engine_state);
        if let Ok(cwd) = env::current_dir()
            && let Some(cwd) = cwd.to_str()
        {
            engine_state.add_env_var("PWD".into(), Value::string(cwd, Span::unknown()));
        }
        nu_std::load_standard_library(&mut engine_state).unwrap();
        engine_state.generate_nu_constant();
        engine_state
    });
    &ENGINE
}

/// Parse Nushell source code into an AST and return both the Block and
/// `StateWorkingSet`, along with the file's starting offset in the span space.
pub fn parse_source<'a>(
    engine_state: &'a EngineState,
    source: &[u8],
    file_path: Option<&Path>,
) -> (Block, StateWorkingSet<'a>, usize) {
    let mut working_set = StateWorkingSet::new(engine_state);

    let (fname, file_buf) = match file_path {
        Some(p) => (p.display().to_string(), p.to_path_buf()),
        None => ("source".to_string(), Path::new("source").to_path_buf()),
    };

    // Get the offset where this file will start in the virtual span space
    let file_offset = working_set.next_span_start();
    // Add the source to the working set's file stack so spans work correctly
    let _file_id = working_set.add_file(&fname, source);
    // Populate `files` to make `path self` command work
    working_set.files = FileStack::with_file(file_buf);
    let block = parse(&mut working_set, Some(&fname), source, false);

    ((*block).clone(), working_set, file_offset)
}

pub struct LintEngine {
    config: Config,
    engine_state: &'static EngineState,
    enabled_rules: Vec<(&'static dyn Rule, LintLevel)>,
}

impl LintEngine {
    #[must_use]
    pub fn new(config: Config, catalog: RuleCatalog) -> Self {
        let enabled_rules = catalog
            .rules
            .iter()
            .map(|rule| (*rule, config.get_lint_level(*rule, catalog.groups)))
            .filter(|(_, lint_level)| *lint_level != LintLevel::Off)
            .collect();
        Self {
            config,
            engine_state: default_engine_state(),
            enabled_rules,
        }
    }

    #[allow(
        clippy::missing_errors_doc,
        reason = "the only error is ReadSourceError, which names the unreadable path"
    )]
    pub fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, ReadSourceError> {
        log::debug!("Linting file: {}", path.display());
        let source = fs::read_to_string(path).map_err(|source| ReadSourceError {
            path: path.to_path_buf(),
            source,
        })?;

        let file_path = fs::canonicalize(path).ok();
        let (block, working_set, file_offset) =
            parse_source(self.engine_state, source.as_bytes(), file_path.as_deref());
        let mut violations = self.lint_parsed(&source, &block, &working_set, file_offset);

        for violation in &mut violations {
            violation.file = Some(path.into());
        }

        violations.sort_by(|a, b| {
            a.file_span()
                .start
                .cmp(&b.file_span().start)
                .then(a.lint_level.cmp(&b.lint_level))
        });
        Ok(violations)
    }

    /// Lint multiple files, optionally in parallel
    ///
    /// Returns a tuple of (violations, `has_errors`) where `has_errors`
    /// indicates if any files failed to be read/parsed.
    #[must_use]
    pub fn lint_files(&self, files: &[PathBuf]) -> Vec<Violation> {
        let violations_mutex = Mutex::new(Vec::new());

        let process_file = |path: &PathBuf| match self.lint_file(path) {
            Ok(violations) => {
                violations_mutex
                    .lock()
                    .expect("Failed to lock violations mutex")
                    .extend(violations);
            }
            Err(e) => {
                log::error!("Error linting {}: {}", path.display(), e);
            }
        };

        if self.config.sequential {
            for path in files {
                log::debug!("Processing file: {}", path.display());
                process_file(path);
            }
        } else {
            files.par_iter().for_each(process_file);
        }

        violations_mutex
            .into_inner()
            .expect("Failed to unwrap violations mutex")
    }

    /// Lint content from standard input
    #[must_use]
    pub fn lint_stdin(&self, source: &str) -> Vec<Violation> {
        let mut violations = self.lint_str(source);
        let source_owned = source.to_string();

        for violation in &mut violations {
            violation.file = Some(SourceFile::Stdin);
            violation.source = Some(source_owned.clone().into());
        }

        violations
    }

    #[must_use]
    pub fn lint_str(&self, source: &str) -> Vec<Violation> {
        let (block, working_set, file_offset) =
            parse_source(self.engine_state, source.as_bytes(), None);
        self.lint_parsed(source, &block, &working_set, file_offset)
    }

    fn lint_parsed(
        &self,
        source: &str,
        block: &Block,
        working_set: &StateWorkingSet,
        file_offset: usize,
    ) -> Vec<Violation> {
        let context = LintContext::new(
            source,
            block,
            self.engine_state,
            working_set,
            file_offset,
            &self.config,
        );

        let mut violations = self.detect_with_fix_data(&context);

        for violation in &mut violations {
            violation.normalize_spans(file_offset);
        }

        let ignore_index = ignore::IgnoreIndex::new(source);
        violations
            .into_iter()
            .filter(|v| {
                let rule_id = v.rule_id.as_deref().unwrap_or("");
                !ignore_index.should_ignore(v.file_span().start, rule_id)
            })
            .collect()
    }

    fn detect_with_fix_data(&self, context: &LintContext) -> Vec<Violation> {
        self.enabled_rules
            .iter()
            .filter_map(|&(rule, lint_level)| {
                let mut violations = rule.check(context);
                for violation in &mut violations {
                    violation.set_rule_id(rule.id());
                    violation.set_lint_level(lint_level.try_into().unwrap());
                    violation.set_doc_url(rule.source_link());
                    violation.set_short_description(rule.short_description());
                    violation.set_diagnostic_tags(rule.diagnostic_tags());
                }

                (!violations.is_empty()).then_some(violations)
            })
            .flatten()
            .collect()
    }
}
