use std::{
    env, fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use ignore::WalkBuilder;
use nu_parser::parse;
use nu_protocol::{
    Span, Value,
    ast::Block,
    engine::{EngineState, FileStack, StateWorkingSet},
};
use rayon::prelude::*;

use crate::{
    LintError,
    config::Config,
    context::LintContext,
    rules::USED_RULES,
    violation::{SourceFile, Violation},
};

/// Parse Nushell source code into an AST and return both the Block and
/// `StateWorkingSet`, along with the file's starting offset in the span space.
pub fn parse_source<'a>(
    engine_state: &'a EngineState,
    source: &[u8],
) -> (Block, StateWorkingSet<'a>, usize) {
    let mut working_set = StateWorkingSet::new(engine_state);
    // Get the offset where this file will start in the virtual span space
    let file_offset = working_set.next_span_start();
    // Add the source to the working set's file stack so spans work correctly
    let _file_id = working_set.add_file("source".to_string(), source);
    // Populate `files` to make `path self` command work
    working_set.files = FileStack::with_file(Path::new("source").to_path_buf());
    let block = parse(&mut working_set, Some("source"), source, false);

    ((*block).clone(), working_set, file_offset)
}

/// Check if a file is a Nushell script (by extension or shebang)
fn is_nushell_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext == "nu")
        || fs::File::open(path)
            .ok()
            .and_then(|file| {
                let mut reader = io::BufReader::new(file);
                let mut first_line = String::new();
                reader.read_line(&mut first_line).ok()?;
                first_line.starts_with("#!").then(|| {
                    first_line
                        .split_whitespace()
                        .any(|word| word.ends_with("/nu") || word == "nu")
                })
            })
            .unwrap_or(false)
}

/// Collect .nu files from a directory, respecting .gitignore files
#[must_use]
pub fn collect_nu_files_from_dir(dir: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(dir)
        .standard_filters(true)
        .build()
        .filter_map(|result| match result {
            Ok(entry) => {
                let path = entry.path().to_path_buf();
                (path.is_file() && is_nushell_file(&path)).then_some(path)
            }
            Err(err) => {
                log::warn!("Error walking directory: {err}");
                None
            }
        })
        .collect()
}

/// Collect all Nushell files to lint from given paths
///
/// For files: includes them if they are `.nu` files or have a nushell shebang
/// For directories: recursively collects `.nu` files, respecting `.gitignore`
#[must_use]
pub fn collect_nu_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .flat_map(|path| {
            if !path.exists() {
                log::warn!("Path not found: {}", path.display());
                return vec![];
            }

            if path.is_file() {
                if is_nushell_file(path) {
                    vec![path.clone()]
                } else {
                    vec![]
                }
            } else if path.is_dir() {
                collect_nu_files_from_dir(path)
            } else {
                vec![]
            }
        })
        .collect()
}

pub struct LintEngine {
    pub(crate) config: Config,
    engine_state: &'static EngineState,
}

impl LintEngine {
    /// Get or initialize the default engine state
    #[must_use]
    pub fn new_state() -> &'static EngineState {
        static ENGINE: LazyLock<EngineState> = LazyLock::new(|| {
            let mut engine_state = nu_cmd_lang::create_default_context();
            engine_state = nu_command::add_shell_command_context(engine_state);
            engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
            engine_state = nu_cli::add_cli_context(engine_state);
            // Required by command `path self`
            if let Ok(cwd) = env::current_dir()
                && let Some(cwd) = cwd.to_str()
            {
                engine_state.add_env_var("PWD".into(), Value::string(cwd, Span::unknown()));
            }

            // Add print command (exported by nu-cli but not added by add_cli_context)
            let delta = {
                let mut working_set = StateWorkingSet::new(&engine_state);
                working_set.add_decl(Box::new(nu_cli::Print));
                working_set.render()
            };
            engine_state
                .merge_delta(delta)
                .expect("Failed to add Print command");

            // Commented out because not needed for most lints and may slow down
            nu_std::load_standard_library(&mut engine_state).unwrap();

            // Set up $nu constant (required for const evaluation at parse time)
            engine_state.generate_nu_constant();

            engine_state
        });
        &ENGINE
    }

    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            engine_state: Self::new_state(),
        }
    }

    /// Lint a file at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub(crate) fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        log::debug!("Linting file: {}", path.display());
        let source = fs::read_to_string(path).map_err(|source| LintError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut violations = self.lint_str(&source);

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

    /// Lint content from stdin
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
        let (block, working_set, file_offset) = parse_source(self.engine_state, source.as_bytes());

        let context = LintContext::new(
            source,
            &block,
            self.engine_state,
            &working_set,
            file_offset,
            &self.config,
        );

        let mut violations = self.detect_with_fix_data(&context);

        // Normalize all spans in violations to be file-relative
        for violation in &mut violations {
            violation.normalize_spans(file_offset);
        }

        violations
    }

    /// Collect violations from all enabled rules
    fn detect_with_fix_data(&self, context: &LintContext) -> Vec<Violation> {
        USED_RULES
            .iter()
            .filter_map(|rule| {
                let lint_level = self.config.get_lint_level(*rule)?;

                let mut violations = rule.check(context);
                for violation in &mut violations {
                    violation.set_rule_id(rule.id());
                    violation.set_lint_level(lint_level);
                    violation.set_doc_url(rule.source_link());
                }

                (!violations.is_empty()).then_some(violations)
            })
            .flatten()
            .collect()
    }
}
