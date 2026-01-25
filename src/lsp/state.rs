use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use lsp_types::{CodeActionOrCommand, Diagnostic, Hover, Range, TextDocumentPositionParams, Uri};

use super::{
    completion::{CodeActionOptions, DisableScope, build_code_actions},
    diagnostic::{LineIndex, extra_labels_to_hint_diagnostics, violation_to_diagnostic},
    docs::build_hover,
};
use crate::{Config, LintEngine, config::find_config_file_from, violation::Violation};

pub struct DocumentState {
    pub content: String,
    pub line_index: LineIndex,
    pub violations: Vec<Violation>,
}

pub struct ServerState {
    engine: LintEngine,
    documents: HashMap<Uri, DocumentState>,
    workspace_root: Option<PathBuf>,
}

impl ServerState {
    pub fn new(config: Config, workspace_root: Option<PathBuf>) -> Self {
        Self {
            engine: LintEngine::new(config),
            documents: HashMap::new(),
            workspace_root,
        }
    }

    /// Reload configuration from workspace or home config file
    pub fn reload_config(&mut self) {
        let config_path = self
            .workspace_root
            .as_ref()
            .and_then(|root| find_config_file_from(root))
            .or_else(|| dirs::home_dir().map(|h| h.join(".nu-lint.toml")));

        let config = config_path
            .and_then(|path| {
                if path.exists() {
                    tracing::info!("Reloading config from {}", path.display());
                    Config::load_from_file(&path).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        self.engine = LintEngine::new(config);
    }

    /// Get the workspace root path
    #[must_use]
    pub fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }

    pub fn lint_document(&mut self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let violations = self.engine.lint_str(content);
        let line_index = LineIndex::new(content);

        let mut diagnostics = vec![];

        for violation in &violations {
            diagnostics.push(violation_to_diagnostic(
                violation,
                content,
                &line_index,
                uri,
            ));

            diagnostics.extend(extra_labels_to_hint_diagnostics(
                violation,
                content,
                &line_index,
            ));
        }

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.to_string(),
                line_index,
                violations,
            },
        );

        diagnostics
    }

    pub fn get_code_actions(&self, uri: &Uri, range: Range) -> Vec<CodeActionOrCommand> {
        let Some(doc_state) = self.documents.get(uri) else {
            return vec![];
        };

        let is_repl = uri.scheme().is_some_and(|s| s.as_str() == "repl");

        let disable_scope = if self.workspace_root.is_some() {
            DisableScope::Workspace
        } else {
            DisableScope::Global
        };

        build_code_actions(
            uri,
            &range,
            doc_state,
            &CodeActionOptions {
                include_ignore: !is_repl,
                disable_scope,
            },
        )
    }

    pub fn get_document(&self, uri: &Uri) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    pub fn has_document(&self, uri: &Uri) -> bool {
        self.documents.contains_key(uri)
    }

    pub fn close_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    /// Get all currently open document URIs
    #[must_use]
    pub fn open_document_uris(&self) -> Vec<Uri> {
        self.documents.keys().cloned().collect()
    }

    /// Get hover documentation for violations at the given position
    pub fn get_hover(&self, params: &TextDocumentPositionParams) -> Option<Hover> {
        let doc_state = self.documents.get(&params.text_document.uri)?;
        let pos = &params.position;

        let violations_at_pos = doc_state.violations.iter().filter(|v| {
            let span = v.file_span();
            let range =
                doc_state
                    .line_index
                    .span_to_range(&doc_state.content, span.start, span.end);
            range.start <= *pos && *pos <= range.end
        });

        build_hover(violations_at_pos)
    }
}
