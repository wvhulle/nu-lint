use std::{
    borrow::Cow,
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use ignore::WalkBuilder;
use nu_parser::parse;
use nu_protocol::{
    ast::Block,
    engine::{EngineState, StateWorkingSet},
};
use rayon::prelude::*;

use crate::{
    LintError, LintLevel, config::Config, context::LintContext, rules::ALL_RULES,
    violation::Violation,
};

/// Parse Nushell source code into an AST and return both the Block and
/// `StateWorkingSet`, along with the file's starting offset in the span space.
fn parse_source<'a>(engine_state: &'a EngineState, source: &[u8]) -> (Block, StateWorkingSet<'a>, usize) {
    let mut working_set = StateWorkingSet::new(engine_state);
    // Get the offset where this file will start in the virtual span space
    let file_offset = working_set.next_span_start();
    // Add the source to the working set's file stack so spans work correctly
    let _file_id = working_set.add_file("source".to_string(), source);
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
    fn default_engine_state() -> &'static EngineState {
        static ENGINE: OnceLock<EngineState> = OnceLock::new();
        ENGINE.get_or_init(|| {
            let mut engine_state = nu_cmd_lang::create_default_context();
            nu_std::load_standard_library(&mut engine_state).unwrap();
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
    pub fn lint_file(&self, path: &Path) -> Result<Vec<Violation>, LintError> {
        log::debug!("Linting file: {}", path.display());
        let source = fs::read_to_string(path).map_err(|source| LintError::Io {
            path: path.to_path_buf(),
            source,
        })?;
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

    /// Lint multiple files, optionally in parallel
    ///
    /// Returns a tuple of (violations, `has_errors`) where `has_errors`
    /// indicates if any files failed to be read/parsed.
    #[must_use]
    pub fn lint_files(&self, files: &[PathBuf]) -> (Vec<Violation>, bool) {
        let violations_mutex = Mutex::new(Vec::new());
        let has_errors = AtomicBool::new(false);

        let process_file = |path: &PathBuf| match self.lint_file(path) {
            Ok(violations) => {
                violations_mutex
                    .lock()
                    .expect("Failed to lock violations mutex")
                    .extend(violations);
            }
            Err(e) => {
                log::error!("Error linting {}: {}", path.display(), e);
                has_errors.store(true, Ordering::Relaxed);
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

        let violations = violations_mutex
            .into_inner()
            .expect("Failed to unwrap violations mutex");
        (violations, has_errors.load(Ordering::Relaxed))
    }

    /// Lint content from stdin
    #[must_use]
    pub fn lint_stdin(&self, source: &str) -> Vec<Violation> {
        let mut violations = self.lint_str(source);

        let stdin_marker: Cow<'static, str> = "<stdin>".to_owned().into();
        let source_content: Cow<'static, str> = source.to_owned().into();

        for violation in &mut violations {
            violation.file = Some(stdin_marker.clone());
            violation.source = Some(source_content.clone());
        }

        violations
    }

    #[must_use]
    pub fn lint_str(&self, source: &str) -> Vec<Violation> {
        let (block, working_set, file_offset) = parse_source(self.engine_state, source.as_bytes());

        let context = LintContext::new(source, &block, self.engine_state, &working_set, file_offset);

        let mut violations = self.collect_violations(&context);

        // Normalize all spans in violations to be file-relative
        for violation in &mut violations {
            violation.normalize_spans(&context);
        }

        violations
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
                    violation.set_rule_id(rule.id);
                    violation.set_lint_level(lint_level);
                    violation.set_doc_url(rule.doc_url);
                }

                (!violations.is_empty()).then_some(violations)
            })
            .flatten()
            .collect()
    }
}
