use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, Range, TextEdit, Uri,
    WorkspaceEdit,
};

use super::{
    actions::DISABLE_RULE_COMMAND,
    diagnostic::{LineIndex, ranges_overlap, violation_to_diagnostic},
    state::DocumentState,
};
use crate::violation::Fix;

fn workspace_edit(uri: &Uri, edits: Vec<TextEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(uri.clone(), edits)])),
        ..Default::default()
    }
}

fn quickfix_kind(rule_id: &str) -> CodeActionKind {
    CodeActionKind::from(format!("quickfix.nu-lint.{rule_id}"))
}

fn ignore_kind(rule_id: &str) -> CodeActionKind {
    CodeActionKind::from(format!("quickfix.nu-lint.ignore.{rule_id}"))
}

fn disable_kind(rule_id: &str) -> CodeActionKind {
    CodeActionKind::from(format!("quickfix.nu-lint.disable.{rule_id}"))
}

pub fn ignore_comment_edit(content: &str, byte_offset: usize, rule_id: &str) -> TextEdit {
    let line_index = LineIndex::new(content);
    let violation_line = line_index.offset_to_line(byte_offset);
    let line_start = line_index.line_start(violation_line);
    let line_content = line_index.line_content(content, violation_line);
    let trimmed_line = line_content.trim_end();

    let edit_offset = line_start + trimmed_line.len();
    let edit_position = line_index.offset_to_position(edit_offset, content);

    let new_text = if trimmed_line.contains("# nu-lint-ignore:") {
        format!(", {rule_id}")
    } else {
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

fn quickfix_action(
    uri: &Uri,
    rule_id: &str,
    fix: &Fix,
    diagnostic: Diagnostic,
    line_index: &LineIndex,
    content: &str,
) -> CodeActionOrCommand {
    let edits = fix
        .replacements
        .iter()
        .map(|r| {
            let span = r.file_span();
            TextEdit {
                range: line_index.span_to_range(content, span.start, span.end),
                new_text: r.replacement_text.to_string(),
            }
        })
        .collect();

    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("{} [{rule_id}]", fix.explanation),
        kind: Some(quickfix_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(workspace_edit(uri, edits)),
        ..Default::default()
    })
}

fn ignore_line_action(
    uri: &Uri,
    rule_id: &str,
    byte_offset: usize,
    diagnostic: Diagnostic,
    content: &str,
) -> CodeActionOrCommand {
    let edit = ignore_comment_edit(content, byte_offset, rule_id);

    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Ignore `{rule_id}` on this line"),
        kind: Some(ignore_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(workspace_edit(uri, vec![edit])),
        ..Default::default()
    })
}

fn disable_rule_action(
    rule_id: &str,
    diagnostic: Diagnostic,
    scope: DisableScope,
) -> CodeActionOrCommand {
    let scope_str = match scope {
        DisableScope::Workspace => "workspace",
        DisableScope::Global => "user",
    };

    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Disable `{rule_id}` in {scope_str}"),
        kind: Some(disable_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        command: Some(Command {
            title: format!("Disable rule '{rule_id}'"),
            command: DISABLE_RULE_COMMAND.to_string(),
            arguments: Some(vec![serde_json::Value::String(rule_id.to_string())]),
        }),
        ..Default::default()
    })
}

#[derive(Clone, Copy)]
pub enum DisableScope {
    Workspace,
    Global,
}

pub struct CodeActionOptions {
    pub include_ignore: bool,
    pub disable_scope: DisableScope,
}

pub fn build_code_actions(
    uri: &Uri,
    range: &Range,
    doc_state: &DocumentState,
    options: &CodeActionOptions,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    if let Some(action) = fix_all_action(uri, doc_state) {
        actions.push(action);
    }

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

    violations_in_range.sort_by(|a, b| {
        a.file_span()
            .start
            .cmp(&b.file_span().start)
            .then_with(|| b.fix.is_some().cmp(&a.fix.is_some()))
    });

    for violation in violations_in_range {
        let span = violation.file_span();
        let rule_id = violation.rule_id.as_deref().unwrap_or("unknown");
        let diagnostic =
            violation_to_diagnostic(violation, &doc_state.content, &doc_state.line_index, uri);

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
                span.start,
                diagnostic.clone(),
                &doc_state.content,
            ));
        }

        actions.push(disable_rule_action(
            rule_id,
            diagnostic,
            options.disable_scope,
        ));
    }

    actions
}

fn fix_all_action(uri: &Uri, doc_state: &DocumentState) -> Option<CodeActionOrCommand> {
    let fixable: Vec<_> = doc_state
        .violations
        .iter()
        .filter(|v| v.fix.is_some())
        .collect();

    // Only show fix all action if there are at least two fixable violations.
    if fixable.len() <= 1 {
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

    edits.sort_by(|a, b| {
        (b.range.start.line, b.range.start.character)
            .cmp(&(a.range.start.line, a.range.start.character))
    });
    edits.dedup_by(|a, b| ranges_overlap(&a.range, &b.range));

    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Fix all auto-fixable problems ({} fixes)", edits.len()),
        kind: Some(CodeActionKind::SOURCE_FIX_ALL),
        diagnostics: Some(diagnostics),
        edit: Some(workspace_edit(uri, edits)),
        is_preferred: Some(false),
        ..Default::default()
    }))
}
