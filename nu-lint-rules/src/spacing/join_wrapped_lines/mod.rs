use std::collections::BTreeSet;

use nu_lint_analysis::{
    LintLevel,
    ast::lexed_statement::{
        LexedStatement, Token, all_lexed_statements, is_quoted_string_token, is_raw_string_token,
        token_text_equals, top_level_statements,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::Span;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;

struct SourceLine {
    line_start: usize,
    trimmed_end: usize,
    next_line_start: usize,
    indent_width: usize,
    character_count: usize,
    last_word_width: usize,
}

impl SourceLine {
    const fn is_blank(&self) -> bool {
        self.line_start + self.indent_width >= self.trimmed_end
    }

    const fn is_filled_by_one_word(&self, longest_line_width: usize) -> bool {
        self.character_count == longest_line_width
            && self.last_word_width * 2 > self.character_count
    }
}

fn parse_source_lines(source: &str) -> Vec<SourceLine> {
    let mut line_start = 0;
    source
        .split_inclusive('\n')
        .map(|line_text| {
            let visible_text = line_text.trim_end();
            let source_line = SourceLine {
                line_start,
                trimmed_end: line_start + visible_text.len(),
                next_line_start: line_start + line_text.len(),
                indent_width: visible_text.len() - visible_text.trim_start().len(),
                character_count: visible_text.chars().count(),
                last_word_width: visible_text
                    .split_whitespace()
                    .next_back()
                    .map_or(0, |last_word| last_word.chars().count()),
            };
            line_start = source_line.next_line_start;
            source_line
        })
        .collect()
}

fn line_index_at(lines: &[SourceLine], file_position: usize) -> usize {
    lines
        .partition_point(|line| line.next_line_start <= file_position)
        .min(lines.len().saturating_sub(1))
}

fn is_wrappable_string_token(token: &Token, context: &LintContext) -> bool {
    is_quoted_string_token(token, context) && !is_raw_string_token(token, context)
}

fn lines_inside_tokens(
    lines: &[SourceLine],
    context: &LintContext,
    is_selected_token: fn(&Token, &LintContext) -> bool,
) -> BTreeSet<usize> {
    let file_offset = context.file_offset();
    all_lexed_statements(context)
        .iter()
        .flat_map(|statement| &statement.statement_tokens)
        .filter(|token| is_selected_token(token, context))
        .flat_map(|token| {
            let first_line = line_index_at(lines, token.span.start - file_offset);
            let last_line = line_index_at(lines, token.span.end.saturating_sub(file_offset + 1));
            (first_line + 1)..=last_line
        })
        .collect()
}

fn ends_with_line_continuation(statement: &LexedStatement, context: &LintContext) -> bool {
    statement
        .statement_tokens
        .last()
        .is_some_and(|last_token| token_text_equals(last_token, context, b"\\"))
}

fn split_statement_lines(lines: &[SourceLine], context: &LintContext) -> BTreeSet<usize> {
    let file_offset = context.file_offset();
    let statements = top_level_statements(context);
    let mut split_lines = BTreeSet::new();
    let mut anchor_indent = None;
    let mut previous_statement: Option<&LexedStatement> = None;

    for statement in &statements {
        let Some(statement_span) = statement.full_span() else {
            continue;
        };
        let start_line = line_index_at(lines, statement_span.start - file_offset);
        let start_indent = lines[start_line].indent_width;
        let continues_previous = previous_statement.is_some_and(|previous| {
            previous.full_span().is_some_and(|previous_span| {
                line_index_at(lines, previous_span.end.saturating_sub(file_offset + 1)) + 1
                    == start_line
            }) && !ends_with_line_continuation(previous, context)
        }) && anchor_indent.is_some_and(|anchor| start_indent > anchor);

        if continues_previous {
            split_lines.insert(start_line);
        } else {
            anchor_indent = Some(start_indent);
        }
        previous_statement = Some(statement);
    }
    split_lines
}

fn global_span(file_start: usize, file_end: usize, context: &LintContext) -> Span {
    let file_offset = context.file_offset();
    Span::new(file_start + file_offset, file_end + file_offset)
}

fn line_join_detection(
    lines: &[SourceLine],
    continuation_line: usize,
    longest_line_width: usize,
    context: &LintContext,
) -> (Detection, LineRepairEdits) {
    let line_before_break = &lines[continuation_line - 1];
    let content_start = lines[continuation_line].line_start + lines[continuation_line].indent_width;
    let joined_span = global_span(line_before_break.trimmed_end, content_start, context);
    let joining_text = if line_before_break.is_filled_by_one_word(longest_line_width) {
        ""
    } else {
        " "
    };

    (
        Detection::from_global_span("Line looks broken by terminal wrapping", joined_span)
            .with_primary_label("wrapped line break"),
        LineRepairEdits {
            line_edits: vec![Replacement::new(joined_span, joining_text.to_string())],
        },
    )
}

fn common_indent_detection(
    lines: &[SourceLine],
    excluded_lines: &BTreeSet<usize>,
    context: &LintContext,
) -> Option<(Detection, LineRepairEdits)> {
    let indented_lines: Vec<&SourceLine> = lines
        .iter()
        .enumerate()
        .filter(|(line_index, line)| !line.is_blank() && !excluded_lines.contains(line_index))
        .map(|(_, line)| line)
        .collect();
    let common_indent = indented_lines
        .iter()
        .map(|line| line.indent_width)
        .min()
        .filter(|&indent_width| indent_width > 0)?;
    let first_line = indented_lines.first()?;

    Some((
        Detection::from_global_span(
            "Every line shares leading whitespace",
            global_span(
                first_line.line_start,
                first_line.line_start + common_indent,
                context,
            ),
        )
        .with_primary_label("common indentation"),
        LineRepairEdits {
            line_edits: indented_lines
                .iter()
                .map(|line| {
                    Replacement::new(
                        global_span(line.line_start, line.line_start + common_indent, context),
                        String::new(),
                    )
                })
                .collect(),
        },
    ))
}

struct LineRepairEdits {
    line_edits: Vec<Replacement>,
}

struct JoinWrappedLines;

impl DetectFix for JoinWrappedLines {
    type FixInput<'a> = LineRepairEdits;

    fn id(&self) -> &'static str {
        "join_wrapped_lines"
    }

    fn short_description(&self) -> &'static str {
        "Code copied from wrapped terminal output contains extra line breaks"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Copying wrapped terminal output adds hard line breaks and indentation. Inside a \
             string they silently change its value, and between arguments they end the command \
             early. The parser reports no error for either. Detection is a heuristic because \
             these lines are also legal code, so the rule is off by default. Apply it to a pasted \
             command with `nu-lint fix --stdin --only join_wrapped_lines`.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/pipelines.html#multi-line-pipelines")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Off
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let source = unsafe { context.source() };
        let lines = parse_source_lines(source);
        let interior_lines = lines_inside_tokens(&lines, context, is_quoted_string_token);
        let wrapped_string_lines = lines_inside_tokens(&lines, context, is_wrappable_string_token)
            .into_iter()
            .filter(|&line_index| lines[line_index].indent_width > 0);
        let continuation_lines: BTreeSet<usize> = wrapped_string_lines
            .chain(split_statement_lines(&lines, context))
            .collect();
        let longest_line_width = lines
            .iter()
            .map(|line| line.character_count)
            .max()
            .unwrap_or_default();
        let lines_outside_dedent = interior_lines.union(&continuation_lines).copied().collect();

        continuation_lines
            .iter()
            .map(|&continuation_line| {
                line_join_detection(&lines, continuation_line, longest_line_width, context)
            })
            .chain(common_indent_detection(
                &lines,
                &lines_outside_dedent,
                context,
            ))
            .collect()
    }

    fn fix(&self, _context: &LintContext, repair_edits: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: "Undo terminal line wrapping".into(),
            replacements: repair_edits.line_edits.clone(),
        })
    }
}

pub static RULE: &dyn Rule = &JoinWrappedLines;
