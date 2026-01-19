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

/// Create a `TextEdit` that inserts an ignore comment before the violation.
///
/// Handles nushell attributes by placing the comment before any `@` attribute
/// lines.
pub fn ignore_comment_edit(content: &str, byte_offset: usize, rule_id: &str) -> TextEdit {
    let line_index = LineIndex::new(content);

    // Find line number containing the violation
    let violation_line = line_index.offset_to_line(byte_offset);

    // Scan backwards to find insertion line (before any attributes)
    let insert_line = find_insert_line_before_attributes(content, &line_index, violation_line);

    // Get byte offset and indentation for the insertion line
    let insert_offset = line_index.line_start(insert_line);
    let indentation: String = content
        .get(insert_offset..)
        .unwrap_or("")
        .chars()
        .take_while(|c| c.is_whitespace() && *c != '\n')
        .collect();

    let insert_position = line_index.offset_to_position(insert_offset, content);

    TextEdit {
        range: Range {
            start: insert_position,
            end: insert_position,
        },
        new_text: format!("{indentation}# nu-lint-ignore: {rule_id}\n"),
    }
}

/// Scan backwards from a line to find the insertion point before any attribute
/// block.
fn find_insert_line_before_attributes(
    content: &str,
    line_index: &LineIndex,
    start_line: usize,
) -> usize {
    let mut insert_line = start_line;

    for line_num in (0..start_line).rev() {
        let line_content = line_index.line_content(content, line_num);
        let trimmed = line_content.trim();

        if trimmed.is_empty() {
            continue; // Skip empty lines between attributes
        }

        if trimmed.starts_with('@') {
            insert_line = line_num; // Move insertion point before this attribute
        } else {
            break; // Non-attribute line, stop scanning
        }
    }

    insert_line
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
        is_preferred: Some(true),
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
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

/// Build a `CodeAction` that disables a rule via command.
pub fn disable_rule_action(rule_id: &str, diagnostic: Diagnostic) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("[{rule_id}] Disable in .nu-lint.toml"),
        kind: Some(quickfix_kind(rule_id)),
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

    // Filter to violations in range, partition by fixable (fixable first)
    let (fixable, non_fixable): (Vec<_>, Vec<_>) = doc_state
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
        .partition(|v| v.fix.is_some());

    // Process fixable violations first, then non-fixable
    for violation in fixable.into_iter().chain(non_fixable) {
        let file_span = violation.file_span();
        let rule_id = violation.rule_id.as_deref().unwrap_or("unknown");

        let diagnostic =
            violation_to_diagnostic(violation, &doc_state.content, &doc_state.line_index, uri);

        // Add fix action if available
        if let Some(fix) = &violation.fix {
            actions.push(quickfix_action(
                uri,
                rule_id,
                fix,
                diagnostic.clone(),
                &doc_state.line_index,
                &doc_state.content,
            ));
        }

        if options.include_ignore {
            actions.push(ignore_line_action(
                uri,
                rule_id,
                file_span.start,
                diagnostic.clone(),
                &doc_state.content,
            ));
        }

        if options.include_disable {
            actions.push(disable_rule_action(rule_id, diagnostic));
        }
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
