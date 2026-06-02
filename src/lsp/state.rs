use std::collections::HashMap;

use lsp_types::{CodeActionOrCommand, Diagnostic, Hover, Range, TextDocumentPositionParams, Uri};

use super::{
    completion::{CodeActionOptions, build_code_actions},
    diagnostic::{LineIndex, extra_labels_to_hint_diagnostics, violation_to_diagnostic},
    docs::build_hover,
};
use crate::{Config, LintEngine, config::load_user_config, violation::Violation};

pub struct DocumentState {
    pub content: String,
    pub line_index: LineIndex,
    pub violations: Vec<Violation>,
}

pub struct ServerState {
    engine: LintEngine,
    documents: HashMap<Uri, DocumentState>,
    is_repl_client: bool,
}

impl ServerState {
    pub fn new(config: Config, is_repl_client: bool) -> Self {
        Self {
            engine: LintEngine::new(config),
            documents: HashMap::new(),
            is_repl_client,
        }
    }

    /// Reload the user-wide XDG config.
    pub fn reload_config(&mut self) {
        self.engine = LintEngine::new(load_user_config());
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

        let is_repl = self.is_repl_client || uri.scheme().is_some_and(|s| s.as_str() == "repl");

        build_code_actions(
            uri,
            &range,
            doc_state,
            &CodeActionOptions {
                include_ignore: !is_repl,
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
