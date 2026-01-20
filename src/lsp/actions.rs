use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, Range, TextEdit, Uri,
    WorkspaceEdit,
};

use super::{
    diagnostic::{LineIndex, ranges_overlap, violation_to_diagnostic},
    server::DISABLE_RULE_COMMAND,
    state::DocumentState,
};
use crate::violation::Fix;

/// Create a `TextEdit` that inserts or appends to an ignore comment inline at
/// the end of the line containing the violation.
pub fn ignore_comment_edit(content: &str, byte_offset: usize, rule_id: &str) -> TextEdit {
    let line_index = LineIndex::new(content);

    // Find line number containing the violation
    let violation_line = line_index.offset_to_line(byte_offset);

    // Get the line content
    let line_start = line_index.line_start(violation_line);
    let line_content = line_index.line_content(content, violation_line);
    let trimmed_line = line_content.trim_end();

    // Check if there's already an ignore comment on this line
    let has_ignore_comment = trimmed_line.contains("# nu-lint-ignore:");

    let edit_offset = line_start + trimmed_line.len();
    let edit_position = line_index.offset_to_position(edit_offset, content);

    let new_text = if has_ignore_comment {
        // Append to existing ignore comment
        format!(", {rule_id}")
    } else {
        // Add a new ignore comment
        format!(" # nu-lint-ignore: {rule_id}")
    };

    TextEdit {
        range: Range {
            start: edit_position,
            end: edit_position,
        },
        new_text,
    }
}

/// Create a hierarchical quickfix kind for a specific rule.
fn quickfix_kind(rule_id: &str) -> CodeActionKind {
    CodeActionKind::from(format!("quickfix.nu-lint.{rule_id}"))
}

/// Build a quickfix `CodeAction` from a violation's fix.
pub fn quickfix_action(
    uri: &Uri,
    rule_id: &str,
    fix: &Fix,
    diagnostic: Diagnostic,
    line_index: &LineIndex,
    content: &str,
) -> CodeActionOrCommand {
    let edits: Vec<TextEdit> = fix
        .replacements
        .iter()
        .map(|r| {
            let file_span = r.file_span();
            TextEdit {
                range: line_index.span_to_range(content, file_span.start, file_span.end),
                new_text: r.replacement_text.to_string(),
            }
        })
        .collect();

    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("[{rule_id}] {}", fix.explanation),
        kind: Some(quickfix_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), edits)])),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

/// Build a `CodeAction` that inserts an ignore comment for a rule.
pub fn ignore_line_action(
    uri: &Uri,
    rule_id: &str,
    byte_offset: usize,
    diagnostic: Diagnostic,
    content: &str,
) -> CodeActionOrCommand {
    let edit = ignore_comment_edit(content, byte_offset, rule_id);

    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("[{rule_id}] Ignore on this line"),
        kind: Some(quickfix_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

/// Build a `CodeAction` that disables a rule via command.
pub fn disable_rule_action(rule_id: &str, diagnostic: Diagnostic) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("[{rule_id}] Disable in workspace"),
        kind: Some(quickfix_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: None,
        command: Some(Command {
            title: format!("Disable rule '{rule_id}'"),
            command: DISABLE_RULE_COMMAND.to_string(),
            arguments: Some(vec![serde_json::Value::String(rule_id.to_string())]),
        }),
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

/// Options for building code actions.
pub struct CodeActionOptions {
    /// Whether to include ignore-line actions.
    pub include_ignore: bool,
    /// Whether to include disable-rule actions.
    pub include_disable: bool,
}

/// Build all code actions for violations in a given range.
pub fn build_code_actions(
    uri: &Uri,
    range: &Range,
    doc_state: &DocumentState,
    options: &CodeActionOptions,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Add source.fixAll action if there are any fixes
    if let Some(action) = fix_all_action(uri, doc_state) {
        actions.push(action);
    }

    // Filter to violations in range, collect and sort by position in document
    let mut violations_in_range: Vec<_> = doc_state
        .violations
        .iter()
        .filter(|v| {
            let span = v.file_span();
            let v_range =
                doc_state
                    .line_index
                    .span_to_range(&doc_state.content, span.start, span.end);
            ranges_overlap(range, &v_range)
        })
        .collect();

    // Sort by position in document first, then by whether it has a fix (fixable
    // first)
    violations_in_range.sort_by(|a, b| {
        let span_a = a.file_span();
        let span_b = b.file_span();
        span_a.start.cmp(&span_b.start).then_with(|| {
            // Within same position, put fixable violations first
            b.fix.is_some().cmp(&a.fix.is_some())
        })
    });

    // Process violations in sorted order, creating action groups per violation
    for violation in violations_in_range {
        let file_span = violation.file_span();
        let rule_id = violation.rule_id.as_deref().unwrap_or("unknown");

        let diagnostic =
            violation_to_diagnostic(violation, &doc_state.content, &doc_state.line_index, uri);

        // Build actions for this violation in order: fix first, then ignore, then
        // disable
        let mut violation_actions = Vec::new();

        // Add fix action if available (highest priority)
        if let Some(fix) = &violation.fix {
            violation_actions.push(quickfix_action(
                uri,
                rule_id,
                fix,
                diagnostic.clone(),
                &doc_state.line_index,
                &doc_state.content,
            ));
        }

        if options.include_ignore {
            violation_actions.push(ignore_line_action(
                uri,
                rule_id,
                file_span.start,
                diagnostic.clone(),
                &doc_state.content,
            ));
        }

        if options.include_disable {
            violation_actions.push(disable_rule_action(rule_id, diagnostic));
        }

        // Add all actions for this violation together
        actions.extend(violation_actions);
    }

    actions
}

/// Build a source.fixAll action that applies all available fixes.
fn fix_all_action(uri: &Uri, doc_state: &DocumentState) -> Option<CodeActionOrCommand> {
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
