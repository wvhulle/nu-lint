use std::collections::BTreeSet;

use nu_lint_analysis::{
    LintLevel,
    ast::{
        lexed_statement::{
            LexedStatement, Token, all_lexed_statements, is_quoted_string_token,
            is_raw_string_token, token_text_equals, top_level_statements,
        },
        source_line::{SourceLine, file_source_lines, line_index_at},
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

enum LineRepair {
    JoinTogether(Span),
    JoinWithSpace(Span),
    RemoveIndentation(Vec<Span>),
}

struct LineRepairFinding {
    finding_detection: Detection,
    line_repair: LineRepair,
}

struct StatementExtent {
    start_line: usize,
    end_line: usize,
    start_indent: usize,
    ends_with_backslash: bool,
}

const fn is_filled_by_one_word(source_line: &SourceLine, longest_line_width: usize) -> bool {
    source_line.character_count == longest_line_width
        && source_line.last_word_width * 2 > source_line.character_count
}

fn is_wrappable_string_token(lexed_token: &Token, lint_context: &LintContext) -> bool {
    is_quoted_string_token(lexed_token, lint_context)
        && !is_raw_string_token(lexed_token, lint_context)
}

fn lines_inside_tokens(
    source_lines: &[SourceLine],
    lint_context: &LintContext,
    is_selected_token: fn(&Token, &LintContext) -> bool,
) -> BTreeSet<usize> {
    all_lexed_statements(lint_context)
        .iter()
        .flat_map(|lexed_statement| &lexed_statement.statement_tokens)
        .filter(|lexed_token| is_selected_token(lexed_token, lint_context))
        .flat_map(|lexed_token| {
            let first_line = line_index_at(source_lines, lexed_token.span.start);
            let last_line = line_index_at(source_lines, lexed_token.span.end.saturating_sub(1));
            (first_line + 1)..=last_line
        })
        .collect()
}

fn statement_extent(
    lexed_statement: &LexedStatement,
    source_lines: &[SourceLine],
    lint_context: &LintContext,
) -> Option<StatementExtent> {
    let first_token = lexed_statement.statement_tokens.first()?;
    let last_token = lexed_statement.statement_tokens.last()?;
    let start_line = line_index_at(source_lines, first_token.span.start);
    Some(StatementExtent {
        start_line,
        end_line: line_index_at(source_lines, last_token.span.end.saturating_sub(1)),
        start_indent: source_lines[start_line].indent_width,
        ends_with_backslash: token_text_equals(last_token, lint_context, b"\\"),
    })
}

fn split_statement_lines(
    source_lines: &[SourceLine],
    lint_context: &LintContext,
) -> BTreeSet<usize> {
    let statement_extents: Vec<StatementExtent> = top_level_statements(lint_context)
        .iter()
        .filter_map(|lexed_statement| statement_extent(lexed_statement, source_lines, lint_context))
        .collect();
    let first_indent = statement_extents
        .first()
        .map_or(0, |first_extent| first_extent.start_indent);

    statement_extents
        .windows(2)
        .scan(first_indent, |anchor_indent, extent_pair| {
            let [previous_extent, next_extent] = extent_pair else {
                return None;
            };
            let continues_previous = previous_extent.end_line + 1 == next_extent.start_line
                && !previous_extent.ends_with_backslash
                && next_extent.start_indent > *anchor_indent;
            if !continues_previous {
                *anchor_indent = next_extent.start_indent;
            }
            Some(continues_previous.then_some(next_extent.start_line))
        })
        .flatten()
        .collect()
}

fn line_join_finding(
    source_lines: &[SourceLine],
    continuation_line: usize,
    longest_line_width: usize,
) -> LineRepairFinding {
    let line_before_break = &source_lines[continuation_line - 1];
    let joined_span = Span::new(
        line_before_break.visible_end,
        source_lines[continuation_line].content_start(),
    );

    LineRepairFinding {
        finding_detection: Detection::from_global_span(
            "Line looks broken by terminal wrapping",
            joined_span,
        )
        .with_primary_label("wrapped line break"),
        line_repair: if is_filled_by_one_word(line_before_break, longest_line_width) {
            LineRepair::JoinTogether(joined_span)
        } else {
            LineRepair::JoinWithSpace(joined_span)
        },
    }
}

fn common_indent_finding(
    source_lines: &[SourceLine],
    excluded_lines: &BTreeSet<usize>,
) -> Option<LineRepairFinding> {
    let indented_lines: Vec<&SourceLine> = (0..source_lines.len())
        .filter(|line_index| {
            !source_lines[*line_index].is_blank() && !excluded_lines.contains(line_index)
        })
        .map(|line_index| &source_lines[line_index])
        .collect();
    let common_indent = indented_lines
        .iter()
        .map(|source_line| source_line.indent_width)
        .min()
        .filter(|&indent_width| indent_width > 0)?;
    let indentation_spans: Vec<Span> = indented_lines
        .iter()
        .map(|source_line| {
            Span::new(
                source_line.line_start,
                source_line.line_start + common_indent,
            )
        })
        .collect();

    Some(LineRepairFinding {
        finding_detection: Detection::from_global_span(
            "Every line shares leading whitespace",
            *indentation_spans.first()?,
        )
        .with_primary_label("common indentation"),
        line_repair: LineRepair::RemoveIndentation(indentation_spans),
    })
}

struct JoinWrappedLines;

impl DetectFix for JoinWrappedLines {
    type FixInput<'a> = LineRepair;

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
        let source_lines = file_source_lines(context);
        let interior_lines = lines_inside_tokens(&source_lines, context, is_quoted_string_token);
        let continuation_lines: BTreeSet<usize> =
            lines_inside_tokens(&source_lines, context, is_wrappable_string_token)
                .into_iter()
                .filter(|&line_index| source_lines[line_index].indent_width > 0)
                .chain(split_statement_lines(&source_lines, context))
                .collect();
        let longest_line_width = source_lines
            .iter()
            .map(|source_line| source_line.character_count)
            .max()
            .unwrap_or_default();
        let lines_outside_dedent = interior_lines.union(&continuation_lines).copied().collect();

        continuation_lines
            .iter()
            .map(|&continuation_line| {
                line_join_finding(&source_lines, continuation_line, longest_line_width)
            })
            .chain(common_indent_finding(&source_lines, &lines_outside_dedent))
            .map(|repair_finding| (repair_finding.finding_detection, repair_finding.line_repair))
            .collect()
    }

    fn fix(&self, _lint_context: &LintContext, line_repair: &Self::FixInput<'_>) -> Option<Fix> {
        let replacements = match line_repair {
            LineRepair::JoinTogether(joined_span) => {
                vec![Replacement::new(*joined_span, String::new())]
            }
            LineRepair::JoinWithSpace(joined_span) => {
                vec![Replacement::new(*joined_span, " ".to_string())]
            }
            LineRepair::RemoveIndentation(indentation_spans) => indentation_spans
                .iter()
                .map(|indentation_span| Replacement::new(*indentation_span, String::new()))
                .collect(),
        };
        Some(Fix {
            explanation: "Undo terminal line wrapping".into(),
            replacements,
        })
    }
}

pub static RULE: &dyn Rule = &JoinWrappedLines;
