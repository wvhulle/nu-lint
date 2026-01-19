use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, DiagnosticTag, Hover,
    HoverContents, MarkupContent, MarkupKind, Range, TextDocumentPositionParams, TextEdit, Uri,
    WorkspaceEdit,
};

use super::{
    diagnostic::{
        LineIndex, extra_labels_to_hint_diagnostics, ignore_comment_edit, ranges_overlap,
        violation_to_diagnostic,
    },
    server::DISABLE_RULE_COMMAND,
};
use crate::{
    Config, LintEngine,
    config::{LintLevel, find_config_file_from},
    rules::groups::groups_for_rule,
    violation::Violation,
};

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

    /// Reload configuration from the workspace config file
    pub fn reload_config(&mut self) {
        let config = self
            .workspace_root
            .as_ref()
            .and_then(|root| {
                find_config_file_from(root).and_then(|path| {
                    log::info!("Reloading config from {}", path.display());
                    Config::load_from_file(&path).ok()
                })
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

        let mut actions = Vec::new();
        let is_repl = uri.scheme().is_some_and(|s| s.as_str() == "repl");

        // Add source.fixAll action if there are any fixes
        if let Some(fix_all) = get_fix_all_action(uri, doc_state) {
            actions.push(fix_all);
        }

        for violation in &doc_state.violations {
            let file_span = violation.file_span();
            let violation_range = doc_state.line_index.span_to_range(
                &doc_state.content,
                file_span.start,
                file_span.end,
            );

            if !ranges_overlap(&range, &violation_range) {
                continue;
            }

            let diagnostic =
                violation_to_diagnostic(violation, &doc_state.content, &doc_state.line_index, uri);

            // Add fix action if available
            if let Some(fix) = &violation.fix {
                let edits: Vec<TextEdit> = fix
                    .replacements
                    .iter()
                    .map(|r| {
                        let file_span = r.file_span();
                        TextEdit {
                            range: doc_state.line_index.span_to_range(
                                &doc_state.content,
                                file_span.start,
                                file_span.end,
                            ),
                            new_text: r.replacement_text.to_string(),
                        }
                    })
                    .collect();

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: fix.explanation.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), edits)])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                }));
            }

            // Skip inline ignore action for REPL content (no persistent file to add comment
            // to)
            if !is_repl && let Some(rule_id) = violation.rule_id.as_deref() {
                let edit = ignore_comment_edit(&doc_state.content, file_span.start, rule_id);
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Ignore '{rule_id}' on this line"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
            if let Some(rule_id) = violation.rule_id.as_deref()
                && self.workspace_root.is_some()
            {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Disable rule '{rule_id}' in .nu-lint.toml"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic]),
                    edit: None,
                    command: Some(Command {
                        title: format!("Disable rule '{rule_id}'"),
                        command: DISABLE_RULE_COMMAND.to_string(),
                        arguments: Some(vec![serde_json::Value::String(rule_id.to_string())]),
                    }),
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
        }

        actions
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

        let markdown: String = doc_state
            .violations
            .iter()
            .filter(|v| {
                let span = v.file_span();
                let range =
                    doc_state
                        .line_index
                        .span_to_range(&doc_state.content, span.start, span.end);
                range.start <= *pos && *pos <= range.end
            })
            .map(violation_to_hover_markdown)
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        (!markdown.is_empty()).then_some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        })
    }
}

/// Get a source.fixAll code action that applies all available fixes
fn get_fix_all_action(uri: &Uri, doc_state: &DocumentState) -> Option<CodeActionOrCommand> {
    let fixable: Vec<_> = doc_state
        .violations
        .iter()
        .filter(|v| v.fix.is_some())
        .collect();

    if fixable.is_empty() {
        return None;
    }

    let diagnostics: Vec<_> = fixable
        .iter()
        .map(|v| violation_to_diagnostic(v, &doc_state.content, &doc_state.line_index, uri))
        .collect();

    let mut edits: Vec<_> = fixable
        .iter()
        .flat_map(|v| &v.fix.as_ref().unwrap().replacements)
        .map(|r| {
            let span = r.file_span();
            TextEdit {
                range: doc_state
                    .line_index
                    .span_to_range(&doc_state.content, span.start, span.end),
                new_text: r.replacement_text.to_string(),
            }
        })
        .collect();

    // Sort by position descending, deduplicate overlapping
    edits.sort_by(|a, b| {
        (b.range.start.line, b.range.start.character)
            .cmp(&(a.range.start.line, a.range.start.character))
    });
    edits.dedup_by(|a, b| ranges_overlap(&a.range, &b.range));

    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Fix all auto-fixable problems ({} fixes)", edits.len()),
        kind: Some(CodeActionKind::SOURCE_FIX_ALL),
        diagnostics: Some(diagnostics),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), edits)])),
            ..Default::default()
        }),
        is_preferred: Some(false),
        ..Default::default()
    }))
}

fn violation_to_hover_markdown(v: &Violation) -> String {
    let mut lines = Vec::new();

    // Header with rule ID and category badges
    if let Some(rule_id) = v.rule_id.as_deref() {
        let categories = groups_for_rule(rule_id);
        let badges = format!("({})", categories.join(", "));
        lines.push(format!("### `{rule_id} `{badges}"));
    }

    // Short description as subtitle
    if let Some(short) = v.short_description {
        lines.push(format!("*{short}*"));
    }

    // Long description
    if let Some(desc) = &v.long_description {
        lines.push(String::new());
        lines.push(desc.clone());
    }

    // Status badges (lint level + diagnostic tags)
    let mut badges = Vec::new();
    match v.lint_level {
        LintLevel::Error => badges.push("Error"),
        LintLevel::Warning => badges.push("Warning"),
        LintLevel::Hint => badges.push("Hint"),
    }
    for tag in &v.diagnostic_tags {
        match *tag {
            DiagnosticTag::UNNECESSARY => badges.push("Unnecessary"),
            DiagnosticTag::DEPRECATED => badges.push("Deprecated"),
            _ => {}
        }
    }
    // Documentation link
    if let Some(url) = v.doc_url {
        lines.push(String::new());
        lines.push(format!("[{url}]({url})"));
    }

    lines.join("\n")
}
