use std::{cmp::Reverse, collections::HashMap};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, Range, TextEdit, Uri,
    WorkspaceEdit,
};
use nu_lint_analysis::{
    ignore::{is_header_line, parse_file_ignore_comment},
    violation::Fix,
};

use super::{
    actions::DISABLE_RULE_COMMAND,
    diagnostic::{LineIndex, ranges_overlap, violation_to_diagnostic},
    state::DocumentState,
};

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

fn ignore_file_kind(rule_id: &str) -> CodeActionKind {
    CodeActionKind::from(format!("quickfix.nu-lint.ignore-file.{rule_id}"))
}

/// Compute the edit that adds `rule_id` to a file-level ignore header.
/// Extends an existing `# nu-lint-ignore-file:` line in the header if one
/// exists; otherwise inserts a new line right after a `#!` shebang (or at
/// line 0 if none).
pub fn ignore_file_edit(content: &str, rule_id: &str) -> TextEdit {
    let line_index = LineIndex::new(content);
    let lines: Vec<&str> = content.lines().collect();

    let existing = lines
        .iter()
        .enumerate()
        .take_while(|(_, line)| is_header_line(line))
        .find_map(|(i, line)| parse_file_ignore_comment(line).map(|_| (i, *line)));

    if let Some((i, line)) = existing {
        let end_offset = line_index.line_start(i) + line.len();
        let pos = line_index.offset_to_position(end_offset, content);
        return TextEdit {
            range: Range {
                start: pos,
                end: pos,
            },
            new_text: format!(", {rule_id}"),
        };
    }

    let has_shebang = lines.first().is_some_and(|l| l.starts_with("#!"));
    let insert_offset = line_index.line_start(usize::from(has_shebang));
    let pos = line_index.offset_to_position(insert_offset, content);
    TextEdit {
        range: Range {
            start: pos,
            end: pos,
        },
        new_text: format!("# nu-lint-ignore-file: {rule_id}\n"),
    }
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
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("{} [{rule_id}]", fix.explanation),
        kind: Some(quickfix_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(workspace_edit(
            uri,
            fix_text_edits(fix, line_index, content),
        )),
        ..Default::default()
    })
}

fn fix_text_edits(fix: &Fix, line_index: &LineIndex, content: &str) -> Vec<TextEdit> {
    fix.replacements
        .iter()
        .map(|replacement| {
            let span = replacement.file_span();
            TextEdit {
                range: line_index.span_to_range(content, span.start, span.end),
                new_text: replacement.replacement_text.to_string(),
            }
        })
        .collect()
}

fn edit_ranges_conflict(first_range: &Range, second_range: &Range) -> bool {
    first_range.start == second_range.start
        || (first_range.start < second_range.end && second_range.start < first_range.end)
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

fn ignore_file_action(
    uri: &Uri,
    rule_id: &str,
    diagnostic: Diagnostic,
    content: &str,
) -> CodeActionOrCommand {
    let edit = ignore_file_edit(content, rule_id);
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Ignore `{rule_id}` in this file"),
        kind: Some(ignore_file_kind(rule_id)),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(workspace_edit(uri, vec![edit])),
        ..Default::default()
    })
}

fn disable_rule_action(rule_id: &str, diagnostic: Diagnostic) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Disable `{rule_id}` in user config"),
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

pub struct CodeActionOptions {
    pub include_ignore: bool,
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
            actions.push(ignore_file_action(
                uri,
                rule_id,
                diagnostic.clone(),
                &doc_state.content,
            ));
        }

        actions.push(disable_rule_action(rule_id, diagnostic));
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

    let mut accepted_edits: Vec<TextEdit> = Vec::new();
    let diagnostics: Vec<_> = fixable
        .into_iter()
        .filter(|violation| {
            let fix_edits = fix_text_edits(
                violation.fix.as_ref().unwrap(),
                &doc_state.line_index,
                &doc_state.content,
            );
            let conflicts_with_accepted = fix_edits.iter().any(|fix_edit| {
                accepted_edits.iter().any(|accepted_edit| {
                    edit_ranges_conflict(&fix_edit.range, &accepted_edit.range)
                })
            });
            if !conflicts_with_accepted {
                accepted_edits.extend(fix_edits);
            }
            !conflicts_with_accepted
        })
        .map(|v| violation_to_diagnostic(v, &doc_state.content, &doc_state.line_index, uri))
        .collect();

    accepted_edits.sort_by_key(|text_edit| Reverse(text_edit.range.start));

    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!(
            "Fix all auto-fixable problems ({} fixes)",
            diagnostics.len()
        ),
        kind: Some(CodeActionKind::SOURCE_FIX_ALL),
        diagnostics: Some(diagnostics),
        edit: Some(workspace_edit(uri, accepted_edits)),
        is_preferred: Some(false),
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use lsp_types::Position;
    use nu_lint_analysis::{
        config::Config,
        span::FileSpan,
        violation::{Detection, Replacement, Violation},
    };

    use super::*;
    use crate::{lsp::state::ServerState, rule_selection::restrict_config_to_selection};

    fn position_offset(line_index: &LineIndex, position: Position) -> usize {
        line_index.line_start(position.line as usize) + position.character as usize
    }

    fn apply_edits_to_source(source_text: &str, text_edits: &[TextEdit]) -> String {
        let line_index = LineIndex::new(source_text);
        let mut descending_edits: Vec<_> = text_edits.iter().collect();
        descending_edits.sort_by_key(|text_edit| Reverse(text_edit.range.start));
        descending_edits.into_iter().fold(
            source_text.to_string(),
            |mut edited_source, text_edit| {
                edited_source.replace_range(
                    position_offset(&line_index, text_edit.range.start)
                        ..position_offset(&line_index, text_edit.range.end),
                    &text_edit.new_text,
                );
                edited_source
            },
        )
    }

    fn only_document_edits(code_action: CodeAction) -> Vec<TextEdit> {
        code_action
            .edit
            .unwrap()
            .changes
            .unwrap()
            .into_values()
            .next()
            .unwrap()
    }

    fn fix_all_result(source_text: &str, selected_rule: &str) -> String {
        let restricted_config =
            restrict_config_to_selection(Config::default(), &[selected_rule.to_string()]).unwrap();
        let mut server_state = ServerState::new(restricted_config, false);
        let document_uri: Uri = "repl:///repl.nu".parse().unwrap();
        server_state.lint_document(&document_uri, source_text);
        let document_state = server_state.get_document(&document_uri).unwrap();
        let Some(CodeActionOrCommand::CodeAction(fix_all)) =
            fix_all_action(&document_uri, document_state)
        else {
            panic!("expected a fix-all code action");
        };
        apply_edits_to_source(source_text, &only_document_edits(fix_all))
    }

    #[test]
    fn fix_all_keeps_every_fix_on_one_line() {
        assert_eq!(
            fix_all_result("ls|first|last", "pipe_spacing"),
            "ls | first | last"
        );
    }

    #[test]
    fn fix_all_keeps_fixes_sharing_a_line_boundary() {
        assert_eq!(
            fix_all_result(
                "^docker run \\\n    --rm \\\n    alpine",
                "shell_line_continuation"
            ),
            "^docker run --rm alpine"
        );
    }

    fn violation_replacing_spans(replaced_spans: &[FileSpan]) -> Violation {
        Violation::from_detected(
            Detection::from_file_span("sample violation", replaced_spans[0]),
            Some(Fix {
                explanation: "sample fix".into(),
                replacements: replaced_spans
                    .iter()
                    .map(|replaced_span| Replacement::with_file_span(*replaced_span, "X"))
                    .collect(),
            }),
            None,
        )
    }

    #[test]
    fn fix_all_drops_whole_fix_on_real_overlap() {
        let source_text = "abcdefghijkl";
        let document_uri: Uri = "repl:///repl.nu".parse().unwrap();
        let document_state = DocumentState {
            content: source_text.to_string(),
            line_index: LineIndex::new(source_text),
            violations: vec![
                violation_replacing_spans(&[FileSpan::new(0, 5)]),
                violation_replacing_spans(&[FileSpan::new(3, 6), FileSpan::new(9, 11)]),
                violation_replacing_spans(&[FileSpan::new(5, 7)]),
            ],
        };
        let Some(CodeActionOrCommand::CodeAction(fix_all)) =
            fix_all_action(&document_uri, &document_state)
        else {
            panic!("expected a fix-all code action");
        };
        assert_eq!(fix_all.title, "Fix all auto-fixable problems (2 fixes)");
        assert_eq!(fix_all.diagnostics.as_ref().unwrap().len(), 2);
        assert_eq!(
            apply_edits_to_source(source_text, &only_document_edits(fix_all)),
            "XXhijkl"
        );
    }

    #[test]
    fn insertions_at_same_position_conflict() {
        let insertion_point = Position::new(0, 3);
        let insertion_range = Range::new(insertion_point, insertion_point);
        assert!(edit_ranges_conflict(&insertion_range, &insertion_range));
    }
}
